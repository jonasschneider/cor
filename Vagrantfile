# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.box = "archlinux-x86_64"

  config.vm.define "arch" do
    config.vm.network "private_network", type: "dhcp", ip: "172.28.128.80"
    config.vm.box = "archlinux-x86_64"
  end
end

# apt-get update && apt-get -y install gdb ruby qemu git-core
# sudo gem install minitest subprocess cucumber
# curl -sSf https://static.rust-lang.org/rustup.sh | sudo sh -s -- --channel=nightly
# cd /vagrant
# cucumber
