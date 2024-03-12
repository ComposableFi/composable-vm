#!/bin/bash

terraform destroy \
    -var "do_token=${DO_PAT}"