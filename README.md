# rust_1pass

1Password CLI wrapper.

This project is work-in-progress.

Goals:

- work cross-platforms
- automatically get the latest release from 1Password release page
- optionally curate the local installed versions similar to `pyenv` and `goenv`
- provide a simple API for the common 1Password operations (sign-in, get-item, etc.)
- is stateless and transient; it doesn't store the user's sensitive value
- fully tested; safe and performant (by using multi-segment downloading approach and
  async)
- clean coding style; source code should be self-explanatory; document the public
  interface
