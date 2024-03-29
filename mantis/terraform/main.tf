resource "digitalocean_droplet" "droplet" {
  name               = "ubuntu-nyc1-node-01"
  image              = "ubuntu-22-04-x64"
  region             = "nyc1"
  size               = "s-1vcpu-1gb"
  ssh_keys = [
    digitalocean_ssh_key.sshkey.id
  ]
  tags = [digitalocean_tag.tag1.id, digitalocean_tag.tag2.id]

  user_data = file("mantis_app.yaml")
}

resource "digitalocean_tag" "tag1" {
  name = "mantis"
}

resource "digitalocean_tag" "tag2" {
  name = "solver"
}


resource "digitalocean_firewall" "firewall" {
  name = "only-22-8080"

  droplet_ids = [digitalocean_droplet.droplet.id]

  # open ssh
  inbound_rule {
    protocol         = "tcp"
    port_range       = "22"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  # open 8000
  inbound_rule {
    protocol         = "tcp"
    port_range       = "8000"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  # open all outgoing tcp
  outbound_rule {
    protocol              = "tcp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }

  # open all outgoing udp
  outbound_rule {
    protocol              = "udp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }

  # open all outgoing icmp
  outbound_rule {
    protocol              = "icmp"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }
}


resource "digitalocean_ssh_key" "sshkey" {
  name       = "my ssh public key"
  public_key = var.ssh_public_key
}

resource "digitalocean_project" "project" {
  name        = "solver"
  description = "Mantis solver"
  purpose     = "Solver related resources"
  environment = "development"
  resources   = [
      "${digitalocean_droplet.droplet.urn}"
    ]
}