from pydantic import BaseModel

from blackbox.osmosis_pools import Model as OsmosisPoolsModel

class AllData(BaseModel):
    osmosis_pools: OsmosisPoolsModel


class OsmosisPoolsResponse(BaseModel):
    pools: OsmosisPoolsModel