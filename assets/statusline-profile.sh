#!/bin/sh
# Usage: sh statusline-profile.sh <profile>
# profile: work | personal
PROFILE="$1"

input=$(cat)

cwd=$(echo "$input" | jq -r '.cwd')
basename=$(basename "$cwd")

git_branch=""
git_dirty=""
if git_branch_raw=$(git -C "$cwd" symbolic-ref --short HEAD 2>/dev/null); then
  git_branch="$git_branch_raw"
  if [ -n "$(git -C "$cwd" status --porcelain 2>/dev/null)" ]; then
    git_dirty="✗"
  fi
fi

model=$(echo "$input" | jq -r '.model.display_name')
used=$(echo "$input" | jq -r '.context_window.used_percentage // empty')

# Colors — use printf to produce actual ESC bytes
esc=$(printf '\033')
green="${esc}[1;32m"
cyan="${esc}[0;36m"
blue="${esc}[1;34m"
red="${esc}[0;31m"
yellow="${esc}[0;33m"
magenta="${esc}[1;35m"
reset="${esc}[0m"

# Profile badge
case "$PROFILE" in
  work)     profile_badge="${blue}[work]${reset} " ;;
  personal) profile_badge="${magenta}[personal]${reset} " ;;
  *)        profile_badge="" ;;
esac

if [ -n "$git_branch" ]; then
  if [ -n "$git_dirty" ]; then
    git_part=" ${blue}git:(${red}${git_branch}${blue}) ${yellow}${git_dirty}${reset}"
  else
    git_part=" ${blue}git:(${red}${git_branch}${blue})${reset}"
  fi
else
  git_part=""
fi

context_part=""
session_info=$(claude-switch usage "$PROFILE" 2>/dev/null)
if [ -n "$session_info" ]; then
  context_part=" ${cyan}${session_info}${reset}"
fi

printf "%s%s%s%s%s%s  %s\n" \
  "${green}➜${reset}  " \
  "$profile_badge" \
  "${cyan}${basename}${reset}" \
  "$git_part" \
  "$context_part" \
  "" \
  "${cyan}${model}${reset}"
