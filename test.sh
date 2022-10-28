function ac() {
  message=$(git branch --show-current): $1
  git commit -m $message
}