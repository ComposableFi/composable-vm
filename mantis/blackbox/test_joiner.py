from blackbox.cvm_runtime.execute import (
    AssetId,
    AssetItem,
    AssetReference,
    AssetReference1,
    Native,
    NetworkId,
)


def test_bases():
    asset = AssetItem(
        asset_id=AssetId("42"),
        local=AssetReference(AssetReference1(native=Native(denom="osmo"))),
        network_id=NetworkId("2"),
    )
    asset = AssetItem(
        asset_id=AssetId("13"),
        local=AssetReference(AssetReference1(native=Native(denom="pica"))),
        network_id=NetworkId("3"),
    )