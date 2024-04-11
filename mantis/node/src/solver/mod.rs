pub mod cows;
pub mod router;
pub mod types;



async fn solve<Decider: Get<bool>>(
    active_orders: Vec<OrderItem>,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    tip: &Tip,
    cvm_glt: Option<cw_cvm_outpost::msg::CvmGlt>,
    router: &String,
) -> Vec<cw_mantis_order::ExecMsg> {
    let cows_per_pair = mantis_node::mantis::solve::find_cows(&active_orders);
    let mut msgs = vec![];
    for pair_solution in cows_per_pair {
        let salt = crate::cvm::calculate_salt(signing_key, tip, pair_solution.ab.clone());
        let cvm_program = if let Some(ref cvm_glt) = cvm_glt {
            let cvm_program = intent_banks_to_cvm_program(
                pair_solution.clone(),
                &active_orders,
                cvm_glt,
                router,
                &salt,
            )
            .await;

            Some(cvm_program)
        } else {
            None
        };

        // would be reasonable to do do cross chain if it solves some % of whole trade
        let route = if let Some(cvm_program) = cvm_program
            && Decider::get()
        {
            Some(CrossChainPart::new(
                cvm_program,
                salt.clone(),
                pair_solution.optimal_price.into(),
            ))
        } else {
            None
        };
        let msg = SolutionSubMsg {
            cows: pair_solution.cows.clone(),
            route,
            timeout: tip.timeout(12),
            optimal_price: pair_solution.optimal_price.into(),
        };
        let msg = cw_mantis_order::ExecMsg::Solve { msg };
        msgs.push(msg);
    }
    msgs
}



async fn intent_banks_to_cvm_program(
    pair_solution: PairSolution,
    all_orders: &Vec<OrderItem>,
    cvm_glt: &cw_cvm_outpost::msg::GetConfigResponse,
    router_api: &String,
    salt: &Vec<u8>,
) -> CvmProgram {
    let (a, b) = mantis_node::mantis::solve::IntentBankInput::find_intent_amount(
        pair_solution.cows.as_ref(),
        all_orders,
        pair_solution.optimal_price,
        cvm_glt,
        pair_solution.ab.clone(),
    );

    log::info!(target:"mantis::solver::", "found for cross chain a: {:?}, b: {:?}", a, b);

    let mut instructions = vec![];

    if a.in_asset_amount.0.gt(&0) {
        let mut a_cvm_route = blackbox::get_route(router_api, a, cvm_glt, salt.as_ref()).await;
        instructions.append(&mut a_cvm_route);
    }
    if b.in_asset_amount.0.gt(&0) {
        let mut b_cvm_route = blackbox::get_route(router_api, b, cvm_glt, salt.as_ref()).await;
        instructions.append(&mut b_cvm_route);
    }
    log::info!(target: "mantis::solver", "built instructions: {:?}", instructions);

    let cvm_program = CvmProgram {
        tag: salt.to_vec(),
        instructions,
    };
    cvm_program
}