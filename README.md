# rust_1pass

1Password CLI wrapper and installer.

This project is work-in-progress.

## Objective

rust_1pass aims to provide a library for Rust applications that want to integrate
1Password to their business logic.

### Why not using the 1Password Rest API

The API token is essentially a single factor authentication. It is less secure than the
CLI's 2-factor authentication (provided that 2-factor is turned on in the Account Setting)
.

## Goals

- work cross-platforms, currently tested on:
  - linux (intel i7 & Amd Ryzen 7)
  - apple m1 max
- automatically get the latest release from 1Password release page
- optionally curate the local installed versions similar to `pyenv` and `goenv`
- provide a simple API for the common 1Password operations (sign-in, get-item, etc.)
- is stateless and transient; it doesn't store the user's sensitive value
- fully tested; safe and performant (by using multi-segment downloading approach and
  async)
- clean coding style; source code should be self-explanatory; document the public
  interface
