#
# Cookbook Name:: delivery_rust
# Recipe:: default
#
# Copyright (C) Chef Software, Inc. 2014
#
include_recipe 'chef-sugar::default'
require 'chef/sugar/core_extensions'

include_recipe "delivery_rust::_prep_builder"

if windows?
  windows_package "rust" do
    source "https://static.rust-lang.org/dist/#{node['delivery_rust']['rust_version']}/rust-nightly-x86_64-pc-windows-gnu.msi"
  end
  include_recipe "omnibus::ruby"
else
  cache_dir = Chef::Config[:file_cache_path]

  remote_file "#{cache_dir}/rustup.sh" do
    source "https://static.rust-lang.org/rustup.sh"
  end

  rust_version = node['delivery_rust']['rust_version']
  rustup_cmd = ["bash",
                "#{cache_dir}/rustup.sh",
                "--channel=nightly",
                "--date=#{rust_version}",
                "--yes"].join(' ')

  rustup_cmd << " --disable-sudo" if platform_family?('mac_os_x')

  execute "install rust and cargo" do
    command rustup_cmd
    not_if { rust_version == current_rust_version }
  end
end

include_recipe "delivery_rust::_openssl"
