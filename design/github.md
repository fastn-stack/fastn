# How Does Github Login Work?

The auth related stuff is in `fastn_core::auth` module.

## Login

To login we send user to `/-/auth/login?provider=github&next=<optional redirect url>`.

The `next` can be used to send the user to arbitrary URL after successful signing.

We use `oauth2` crate for authentication with github.

## Callback URL

The callback URL is 

## CSRF Token

Are we CSRF safe? We are generating a CSRF token using `oauth2::CsrfToken::new_random` in 
`fastn_core::auth::github::login()`, but we are not checking it in `fastn_core::auth::github::callback()`. I think
we aught to, else we may be susceptible to CSRF. Not sure how someone can use CSRF in this context, but given
the library supports should too.

How would we verify? Easiest thing would be to store it in a cookie. This is what Django does, stores CSRF token in
cookie, and verifies that tokens match on POST request etc. 

