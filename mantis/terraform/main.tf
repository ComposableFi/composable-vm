resource "digitalocean_droplet" "node" {
  name               = "ubuntu-nyc1-node-01"
  image              = "ubuntu-22-04-x64"
  region             = "nyc1"
  size               = "s-1vcpu-1gb"
}

resource "digitalocean_project" "project" {
  name        = "solver"
  description = "Mantis solver"
  purpose     = "Solver related resources"
  environment = "development"
  resources   = [
      "${digitalocean_droplet.node.urn}"
    ]
}