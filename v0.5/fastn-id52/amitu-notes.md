lets focus on fastn-id52 for a while and review its generate command, it should
by default store the generated key in keyring, so if -k should be also
supported, --keyring, and if it is not passed it should be assumed the default.
if -k is passed we will generate the key store its secret in keyring and print
its public key on stdout. -f can overwrite this behaviour.

--------


review ../kulfi to see how we use keyring. keys might already be stored in
keyring, those keys created via malai, so our keys should be stored in exactly
the format kulfi/malai uses. lets write a note on keyring first.


----

in general a fallback when reading, using password and storing hex may be better
design choice, as then user can easily see the private key using their password
manager │
│ tool as passwords are shown (after explicit user request), but secret bytes
are not and even if they are they are hard to copy paste being binary. we should
continue to │
│ read from secret but when creating keyring entries we should store password.
add this note. 
