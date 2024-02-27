    def simulate_route(parent_result, parent_venue):
        """
        Using models of protocols goes over them and generates final true output
        """
        if parent_venue.children:
            subs = []
            for child in parent_venue.children:
                venue = data.venue_by_index(child.venue_index)
                next_parent = (
                    Exchange(
                        in_asset_amount=math.ceil(child.in_amount),
                        out_asset_id=child.in_asset_id,
                        in_asset_id=parent_result.out_asset_id,
                        out_asset_amount=int(sum([x.in_amount for x in child.children])),
                        pool_id=str(venue.pool_id),
                        next=subs,
                    )
                    if isinstance(venue, AssetPairsXyk)
                    else Spawn(
                        in_asset_amount=math.ceil(child.in_amount),
                        out_asset_id=child.in_asset_id,
                        in_asset_id=parent_result.out_asset_id,
                        out_asset_amount = int(sum([x.in_amount for x in child.children])),
                        next=subs,
                    )
                )
                sub = simulate_route(next_parent, child)
                subs.append(sub)
            parent_result.next = subs
        return parent_result

    parent = SingleInputAssetCvmRoute(
        out_asset_id=input.in_token_id, out_asset_amount=input.in_amount, next=[]
    )
    return simulate_route(parent, route_start)