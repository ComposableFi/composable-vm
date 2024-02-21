# given `ExtendedCvmRegistry` and raw `AllData`, and user `Input`, produced oracalized data with assets and venues route level reachable by user


from blackbox.cvm_indexer import ExtendedCvmRegistry
from pydantic import BaseModel

class Oracle(BaseModel):
    def from_usd(cvm: ExtendedCvmRegistry):
        """_summary_
            Builds USD oracle from data.
        """
        pass
    
    def for_simulation():
        """
        Makes data exactly as it handled by simulation
        """
        pass