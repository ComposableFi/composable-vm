// SPDX-License-Identifier: MIT
pragma solidity ^0.8.14;
pragma experimental ABIEncoderV2;

import "openzeppelin-contracts/access/Ownable.sol";
import "openzeppelin-contracts/token/ERC20/IERC20.sol";

import "./Executor.sol";
import "./interfaces/IRouter.sol";
import "./interfaces/IIbcBridge.sol";

contract Router is Ownable, IRouter {
    // network => account => salt
    mapping(uint128 => mapping(bytes => mapping(bytes => address))) public userExecutor;

    mapping(address => Bridge) public bridgesInfo;
    // TODO ? do we have only one bridge per network and security
    mapping(uint128 => mapping(BridgeSecurity => address)) public bridges;
    mapping(uint256 => address) public assets;
    mapping(address => uint256) public assetIds;

    event InstanceCreated(uint128 networkId, bytes user, bytes salt, address instance);

    event AddOwners(address sender, uint128 networkId, BridgeSecurity security, address[] owners);

    event RemoveOwners(address sender, uint128 networkId, BridgeSecurity security, address[] owners);

    event Spawn(
        bytes account,
        uint128 networkId,
        BridgeSecurity security,
        bytes salt,
        bytes spawnedProgram,
        address[] assetAddresses,
        uint256[] amounts
    );

    modifier onlyBridge() {
        require(bridgesInfo[msg.sender].security != BridgeSecurity(0));
        _;
    }

    constructor() {
        // enable trustless bridge;
    }

    function getAsset(uint256 assetId) external view returns (address) {
        return assets[assetId];
    }

    function getAssetIdByLocalId(address asset) external view returns(uint256) {
        return assetIds[asset];
    }

    function getBridge(uint128 networkId, BridgeSecurity security) external view returns (address) {
        return bridges[networkId][security];
    }

    function registerAsset(address assetAddress, uint128 assetId) external onlyOwner {
        require(assetAddress != address(0), "Router: invalid address");
        assets[assetId] = assetAddress;
        assetIds[assetAddress] = assetId;
    }

    function unregisterAsset(uint128 assetId) external onlyOwner {
        delete assetIds[assets[assetId]];
        delete assets[assetId];
    }

    function registerBridge(
        address bridgeAddress,
        BridgeSecurity security,
        uint128 networkId
    ) external onlyOwner {
        require(bridges[networkId][security] == address(0), "Router: this type of bridge already registered");
        require(bridgeAddress != address(0), "Router: invalid address");
        require(bridgesInfo[bridgeAddress].security == BridgeSecurity(0), "Router: bridge already enabled");
        require(security != BridgeSecurity(0), "Router: should not disable bridge while registering bridge");
        bridgesInfo[bridgeAddress].security = security;
        bridgesInfo[bridgeAddress].networkId = networkId;
        bridges[networkId][security] = bridgeAddress;
    }

    function unregisterBridge(address bridgeAddress) external onlyOwner {
        require(bridgesInfo[bridgeAddress].security != BridgeSecurity(0), "Router: bridge already disabled");
        delete bridges[bridgesInfo[bridgeAddress].networkId][bridgesInfo[bridgeAddress].security];
        bridgesInfo[bridgeAddress].security = BridgeSecurity(0);
        bridgesInfo[bridgeAddress].networkId = 0;
    }

    //// TODO ? is the bridge who's gonna to provide internetwork assets transfer?
    function _provisionAssets(
        address payable executorAddress,
        address[] memory erc20AssetList,
        uint256[] memory amounts
    ) internal {
        require(
            erc20AssetList.length == amounts.length,
            "Router: asset list size should be equal to amount list size"
        );
        if (msg.value > 0) {
            bool sent = executorAddress.send(msg.value);
            require(sent, "Failed to send Ether");
        }
        for (uint256 i = 0; i < erc20AssetList.length; i++) {
            IERC20(erc20AssetList[i]).transferFrom(msg.sender, executorAddress, amounts[i]);
        }
    }

    function runProgram(
        Origin memory origin,
        bytes memory salt,
        bytes memory program,
        address[] memory _assets,
        uint256[] memory _amounts
    ) public override payable onlyBridge returns (bool){
        // a program is a result of spawn function, pull the assets from the bridge to the executor
        address payable executorAddress = getOrCreateExecutor(origin, salt);
        _provisionAssets(executorAddress, _assets, _amounts);

        IExecutor(executorAddress).interpret(program, msg.sender);
        return true;
    }

    function createExecutor(Origin memory origin, bytes memory salt) public returns(address payable) {
        address executorAddress = userExecutor[origin.networkId][origin.account][salt];
        require(executorAddress == address(0), "Executor already exists");
        return getOrCreateExecutor(origin, salt);
    }

    function getOrCreateExecutor(Origin memory origin, bytes memory salt) public returns (address payable) {
        address executorAddress = userExecutor[origin.networkId][origin.account][salt];
        if (executorAddress == address(0)) {
            //executorAddress = address(new Executor(networkId, account));
            require(
                bridgesInfo[msg.sender].security == BridgeSecurity.Deterministic,
                "For creating a new executor, the sender should be a deterministic bridge"
            );
            executorAddress = address(new Executor(origin, address(this), salt));
            userExecutor[origin.networkId][origin.account][salt] = executorAddress;

            emit InstanceCreated(origin.networkId, origin.account, salt, executorAddress);
        }
        return payable(executorAddress);
    }

    function emitSpawn(
        bytes memory account,
        uint128 networkId,
        BridgeSecurity security,
        bytes memory salt,
        bytes memory spawnedProgram,
        address[] memory assetAddresses,
        uint128[] memory _assetIds,
        uint256[] memory amounts
    ) override external {
        address payable executorAddress = getOrCreateExecutor(Origin(networkId, account), IExecutor(msg.sender).salt());
        require(executorAddress == msg.sender, "Router: sender is not an executor address");
        emit Spawn(account, networkId, security, salt, spawnedProgram, assetAddresses, amounts);
        if (security == BridgeSecurity.Deterministic) {
            // send through ibc
            IIbcBridge(bridges[networkId][security]).sendProgram(account, networkId, salt, spawnedProgram, _assetIds, amounts);
        }
    }
}
