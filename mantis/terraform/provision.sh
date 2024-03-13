#!/bin/bash

terraform init

terraform plan \
    -var "do_token=${DO_PAT}" \
    -var-file="terraform.tfvars"

terraform apply \
    -var "do_token=${DO_PAT}" \
    -var-file="terraform.tfvars"