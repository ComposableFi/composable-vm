locals {
  droplet_id                   = "${digitalocean_droplet.droplet.id}"
  droplet_ipv4_address         = "${digitalocean_droplet.droplet.ipv4_address}"
  droplet_ipv4_address_private = "${digitalocean_droplet.droplet.ipv4_address_private}"
  droplet_ipv6_address         = "${digitalocean_droplet.droplet.ipv6_address}"
  droplet_region               = "${digitalocean_droplet.droplet.region}"
  droplet_name                 = "${digitalocean_droplet.droplet.name}"
  droplet_size                 = "${digitalocean_droplet.droplet.size}"
  droplet_image                = "${digitalocean_droplet.droplet.image}"
  droplet_tags                 = "${digitalocean_droplet.droplet.tags}"
}

output "droplet_id" {
  description = "List of IDs of Droplets"
  value       = local.droplet_id
}

output "name" {
  description = "List of names of Droplets"
  value       = local.droplet_name
}

output "ipv4_address" {
  description = "List of public IPv4 addresses assigned to the Droplets"
  value       = local.droplet_ipv4_address
}

output "ipv4_address_private" {
  description = "List of private IPv4 addresses assigned to the Droplets, if applicable"
  value       = local.droplet_ipv4_address_private
}

output "region" {
  description = "List of regions of Droplets"
  value       = local.droplet_region
}

output "size" {
  description = "List of sizes of Droplets"
  value       = local.droplet_size
}

output "tags" {
  description = "List of tags of Droplets"
  value       = local.droplet_tags
}