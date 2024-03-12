# Provision Solver

## Digital Ocean

Generate PAT in Digital Ocean's console panel:
- In the left menu, click API, which takes you to the Applications & API page on the Tokens tab. In the Personal access tokens section, click the Generate New Token button.

[How to Create a Personal Access Token](https://docs.digitalocean.com/reference/api/create-personal-access-token/)

export your Digital Ocean's Personal Access token
```bash
export DO_PAT="your_personal_access_token"
```

### Provision

```bash
./provision.sh
```

At the end of provisioning script you wil find some info about your deployment:
```
Apply complete! Resources: 3 added, 1 changed, 1 destroyed.

Outputs:

droplet_id = "406459888"
ipv4_address = "134.122.118.208"
ipv4_address_private = "10.116.0.3"
name = "ubuntu-nyc1-node-01"
region = "nyc1"
size = "s-1vcpu-1gb"
tags = toset([
  "mantis",
  "solver",
])
```

### Access to droplet

```bash
ssh -i path_to_prv_file root@<ip_of_droplet>
```

```bash
❯ ssh -i ~/.ssh/do_test_id_rsa root@134.122.118.208
Welcome to Ubuntu 22.04.2 LTS (GNU/Linux 5.15.0-67-generic x86_64)

 * Documentation:  https://help.ubuntu.com
 * Management:     https://landscape.canonical.com
 * Support:        https://ubuntu.com/advantage

  System information as of Tue Mar 12 10:21:30 UTC 2024

  System load:  0.0               Users logged in:       0
  Usage of /:   6.9% of 24.05GB   IPv4 address for eth0: 134.122.118.208
  Memory usage: 21%               IPv4 address for eth0: 10.10.0.6
  Swap usage:   0%                IPv4 address for eth1: 10.116.0.3
  Processes:    96

Expanded Security Maintenance for Applications is not enabled.

17 updates can be applied immediately.
13 of these updates are standard security updates.
To see these additional updates run: apt list --upgradable

Enable ESM Apps to receive additional future security updates.
See https://ubuntu.com/esm or run: sudo pro status

The list of available updates is more than a week old.
To check for new updates run: sudo apt update


The programs included with the Ubuntu system are free software;
the exact distribution terms for each program are described in the
individual files in /usr/share/doc/*/copyright.

Ubuntu comes with ABSOLUTELY NO WARRANTY, to the extent permitted by
applicable law.

root@ubuntu-nyc1-node-01:~#
```