_default:
  just --list

deploy:
  fly deploy --remote-only
