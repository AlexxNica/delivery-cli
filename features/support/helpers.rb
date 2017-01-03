#
# Helper methods
#

# Mock a custom config.json
def custom_config
<<EOF
{
  "version": "2",
  "build_cookbook": {
    "path": ".delivery/build_cookbook",
    "name": "build_cookbook"
  },
  "skip_phases": [ "smoke", "security", "syntax", "unit", "quality" ],
  "build_nodes": {},
  "delivery-truck": {
    "publish": {
      "chef_server": true
    }
  },
  "dependencies": []
}
EOF
end

# Mock a project.toml that has missing phases
def incomplete_project_toml
<<EOF
[local_phases]
unit = "echo 'only unit is configured'"
EOF
end

def invalid_project_toml
<<EOF
[local_phasez]
lint = "echo 'This file is wrong, we have typos'"
EOF
end

# Mock a project.toml
def project_toml
<<EOF
[local_phases]
unit = "echo 'This is a cool unit test'"
lint = "cookstyle"
syntax = "foodcritic ."
provision = "chef exec kitchen create"
deploy = "chef exec kitchen converge"
smoke = "chef exec kitchen verify"
cleanup = "chef exec kitchen destroy"
EOF
end

def valid_cli_toml
<<EOF
api_protocol = "https"
enterprise = "ent"
git_port = "8989"
organization = "org"
pipeline = "master"
server = "server.test"
user = "user"
EOF
end

def remote_project_toml
<<EOF
[local_phases]
unit = "echo REMOTE-UNIT"
lint = "echo REMOTE-LINT"
syntax = "echo REMOTE-SYNTAX"
provision = "echo REMOTE-PROVISION"
deploy = "echo REMOTE-DEPLOY"
smoke = "echo REMOTE-SMOKE"
cleanup = "echo REMOTE-CLEANUP"
EOF
end

def project_toml_with_remote_file(url)
<<EOF
remote_file = "#{url}"
[local_phases]
unit = ""
lint = ""
syntax = ""
provision = ""
deploy = ""
smoke = ""
cleanup = ""
EOF
end

# Mock a build_cookbook.rb that doesn't generate a config.json
def build_cookbook_rb
<<EOF
context = ChefDK::Generator.context
delivery_project_dir = context.delivery_project_dir
dot_delivery_dir = File.join(delivery_project_dir, ".delivery")
directory dot_delivery_dir
build_cookbook_dir = File.join(dot_delivery_dir, "build_cookbook")
directory build_cookbook_dir
template "\#{build_cookbook_dir}/metadata.rb" do
  source "build_cookbook/metadata.rb.erb"
  helpers(ChefDK::Generator::TemplateHelper)
  action :create_if_missing
end
template "\#{build_cookbook_dir}/Berksfile" do
  source "build_cookbook/Berksfile.erb"
  helpers(ChefDK::Generator::TemplateHelper)
  action :create_if_missing
end
directory "\#{build_cookbook_dir}/recipes"
%w(default deploy functional lint provision publish quality security smoke syntax unit).each do |phase|
  template "\#{build_cookbook_dir}/recipes/\#{phase}.rb" do
    source 'build_cookbook/recipe.rb.erb'
    helpers(ChefDK::Generator::TemplateHelper)
    variables phase: phase
    action :create_if_missing
  end
end
EOF
end

# Mock a cli.toml config
# Starts a server on 8080 so the git port is also 8080,
# so don't be surprised to see addresses like 127.0.0.1:8080:8080
def basic_delivery_config
<<EOF
git_port = "8080"
pipeline = "master"
user = "dummy"
server = "127.0.0.1:8080"
enterprise = "dummy"
organization = "dummy"
EOF
end

# Mock default delivery config.json
def default_delivery_config
<<EOF
  {
    "version": "2",
    "build_cookbook": {
      "path": ".delivery/build_cookbook",
      "name": "build_cookbook"
    },
    "skip_phases": [],
    "build_nodes": {},
    "dependencies": []
  }
EOF
end

# Mock basic git config
def basic_git_config
<<EOF
[config]
EOF
end

def additional_gen_recipe
<<EOF
file "\#{build_cookbook_dir}/test_file" do
  content 'THIS IS ONLY A TEST.'
end
EOF
end

# Relative path of a temporal directory
def tmp_relative_path
  @tmp_relative_dir ||= '../tmp'
  step %(a directory named "#{@tmp_relative_dir}")
  @tmp_relative_dir
end

# Absolute path of a temporal directory
def tmp_expanded_path
  @tmp_expanded_dir ||= expand_path('../tmp')
end
