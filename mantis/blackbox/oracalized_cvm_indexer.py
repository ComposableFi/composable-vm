# given `ExtendedCvmRegistry` and raw `AllData`, and user `Input`, produced oracalized data with assets and venues route level reachable by user


from blackbox.cvm_indexer import ExtendedCvmRegistry
from pydantic import BaseModel

class Oracle(BaseModel):
    def from_usd(cvm: ExtendedCvmRegistry):
        pass
    
    def for_simulation():
        pass
    def scale_in():
        pass


#     def to_cvm():
#         """_summary_
#         prodduce non CVMed XYK
#         prodcue from CVM REG all IN OUT PAIRS of assets in good format
#         filter out only CVMed with known assets
#         replace with ids in XYK
#         generate Transfers for all, find ga for comoss_chain_of each
#         normalize all assets to oracle if ALL assets has USD.

#         Produced CVMed data from raw chain data consumable by simulation
#         """
# ExtendedCvmRegistry
#         pass
