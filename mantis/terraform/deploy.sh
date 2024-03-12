#!/bin/bash

terraform init

terraform plan -var "do_token=${DO_PAT}"

terraform apply -var "do_token=${DO_PAT}"