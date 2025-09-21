pub fn generate_shell_integration(shell_type: &str) -> String {
    match shell_type {
        "zsh" => generate_zsh_integration(),
        "bash" => generate_bash_integration(),
        "fish" => generate_fish_integration(),
        _ => format!("Unsupported shell type: {shell_type}"),
    }
}

fn generate_zsh_integration() -> String {
    r#"# Zsh integration for ff fuzzy finder
# Source this script to enable ff integration

# Function to run ff with piped input
ff() {
  if [ $# -eq 0 ]; then
      # No arguments, read from stdin and save to temp file
      local temp_file="/tmp/ff-$$.tmp"
      cat > "$temp_file"
      if [ ! -s "$temp_file" ]; then
          rm -f "$temp_file"
          echo "No input provided" >&2
          return 1
      fi
      ff-bin "$temp_file"
      local exit_code=$?
      rm -f "$temp_file"
      return $exit_code
  else
      # Arguments provided, use regular ff-bin
      ff-bin "$@"
  fi
}"#
    .to_string()
}

fn generate_bash_integration() -> String {
    r#"# Bash integration for ff fuzzy finder
# Source this script to enable ff integration

# Function to run ff with piped input
ff() {
  if [ $# -eq 0 ]; then
      # No arguments, read from stdin and save to temp file
      local temp_file="/tmp/ff-$$.tmp"
      cat > "$temp_file"
      if [ ! -s "$temp_file" ]; then
          rm -f "$temp_file"
          echo "No input provided" >&2
          return 1
      fi
      ff-bin "$temp_file"
      local exit_code=$?
      rm -f "$temp_file"
      return $exit_code
  else
      # Arguments provided, use regular ff-bin
      ff-bin "$@"
  fi
}
"#
    .to_string()
}

fn generate_fish_integration() -> String {
    r#"# Fish integration for ff fuzzy finder
# Source this script to enable ff integration

# Function to run ff with piped input
function ff
  if test (count $argv) -eq 0
      # No arguments, read from stdin and save to temp file
      set temp_file "/tmp/ff-$$.tmp"
      cat > "$temp_file"
      if test ! -s "$temp_file"
          rm -f "$temp_file"
          echo "No input provided" >&2
          return 1
      end
      ff-bin "$temp_file"
      set exit_code $status
      rm -f "$temp_file"
      return $exit_code
  else
      # Arguments provided, use regular ff-bin
      ff-bin $argv
  end
end
"#
    .to_string()
}
