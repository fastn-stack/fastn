/Users/amitu/Projects/kulfi/malai/src/expose_http.rs is how we listen on one
endpoint. we are going to have an endpoint for every entity we have, so each
must run in parallel as an async task. but we have offline/online complication,
so we need a way for us to tell these offline tasks to stop working if an entity
goes offline, and to spawn new tasks when an offline entity goes online or a new
entity is created. how to design this, show plan first

-------

the logic of what will happen in this project is not specified, the http code i
showed was for reference on how endpoint/iroh connection loop works. actually
the action handler that should be called on each messge will depend on the kind
of entity. we currently have a generic entity, but soon we will have specific
entities like Account, Device, Rig etc, which will keep a handle to common
.entry params, and specify account / device specific data. so the first question
is how do we keep track of entry type. when an entry is created it is stored in
uninitialised state, and can be considered offline. we then have method like
entry.initialise_as_account(), which will do account specific initialisation of
the entry (may run account specific migration, create account specific
files/config etc). so lets review our core types first to see how will it all
work

------

we can run one loop per entity kind, like one function for handling accounts and
another for devices, so we do not have to do an inner match etc. similarly we
have to send messages back as well, and i want to have diff queues for diff
types so i can have account type encoded in queue data type itself, which gives
me compile time safety, so I do not have a single Message, which has enums for
each kind, as then diff kind of messages (device message vs account message) in
same type.


--------

let me do a simplification, no entity is created uninitialised, so we do not
need generic entity manager at all, as we can create account / device manager
separately, and our initial loader will provide methods to read all accounts
and all devices

---------

one more rule; there can be only one rig per fastn_home

--------

rig is not optional for a node

-------

actually very soon we will have http server in the mix, we will run a single
http server, and it will accept requests for all entities, <id52>
.localhost:<port>, and it will have to call entity specific methods. i would
prefer even the router being used as entity type specific as each entity type
may have diff set of urls. so how do we design that both iroh and http loops
look quite similar, and there is some beuty to deisgn

-----

so if the http request came for an id52 which is not our account/device/rig id52
but belongs to someone else, we want to proxy that http request over iroh, so
iroh side as well can get incoming http proxy requests that it is supposed to
route to http server on its side, preferably without creating actual http
request or even hyper specific data, maybe some our own http request/response
abstraction that we send over iroh or pass to our http router.


-----

so another rule, when we are making outgoing iroh connection we have to pick
which of our endpoint we are going to use as we have one endpoint per entity. so
lets understand two kinds of relations, (ignore rig for this discussion), we
have account, and account when it makes a connection to device, it only makes a
connection to the device it owns, and each device can have exactly one owner,
and a device ever only connects with account and never with any other entity.
and now lets talk account to account, each account can have one or more aliases,
the id52 we used for the account is actually the first alias, one can argue our
account modelling is wrong, and since folder name for account is its first
alias, but there is nothing special about the first alias, tomorrow we can have
any number of aliases, basically say i have two friends, i will have two entries
in my fastn_account table, and say i have a different alias for both friends, so
in the row we will also mention which of our alias this account knows about.
whenever a new connection comes to us, we put it fastn_account table, and store
our endpoint that it connected to us via, as the alias. before code lets create
a md file explaining the requirements / explain the core concepts of fastn


----


**Storage**: `{fastn_home}/rig.*` files and `rig.db` - lets create a folder for
rig. one thing that is coming is file serving, so each entity, and this is
generic feature across all entry types including rig, is that each entity can
serve files in the entity, and they can store wasm files and we will run wasm
too, and those static files can contain templated files .fhtml, which will be
rendered using some templating library.


---

also we are going to have automerge documents, these would be stored in sqlite
and will use automerge to sync state, documents will be owned by accounts and
synced with devices owned by the owning account, or with peers based on who the
document is shared with, the document share relationship will also be stored
in sqlite.

and then we have email. each account can store emails. we can send emails peer
to peer via fastn network, and account entity is the main source of truth of
emails, and emails are not synced with anything, they are sent to each other
via peer to peer but each mail serving account simply stores the mails in a
folder named mails, where it stores each incoming <username>@<id52> username
folder, and in that folder we store more folders as fastn mail will expose
IMAP and SMPT servers so regular mail clients can send mails to each other via
fastn peer to peer


----

`{username}@{id52}/` - Per-sender email folders is vague-ish, lets make it clear
that every alias gets an email domain, @<alias>. and any <username>@<alias> is a
valid email address as long as <username> is sane. also across all aliases, same
username folder is created, so amitu@alias-1 and amitu@alias-2 mails will be
stored in mails/amitu folder.

further you have further made primary alias special compared to other aliases. i
want all aliases to be equal as if we have primary we are going to accidentially
use primary when we should have picked alias properly, leaving to accidental
privacy discolure


----

lets talk about device to account browsing. device to device browsing is
impossible. we do not want other accounts to ever know our device id52. so
when a device wants to browser a foreign account, for if it wanted to browse
its owner account, it would use the device id52, that is not meant to be private
from the owner!. so for non owner accounts, device can browse those accounts in
private mode or alias mode. in private mode the device will simply create a new
id52 pair, it can use such temporary id52 for browsing across accounts to not
have to create too many id52 and slowing down connection setup time as it takes
a bit of time to do that, so first create id52, some latency, and then actual
connection with target id52, another latency, former can be avoided if we reused
browsing id52. anyways, so that is anonymous browsing, but sometimes it makes
sense to browse as the account, like so you appear logged in and can access
shared with you documents etc, so we will still use browsing id52, but when
sending http requests we will also send a signature we got from the
owning-account-alias i want to act on behalf of. to get this signature the
device will have to pass the browsing id52 to the account via p2p, and get a
signed message back saying assume this fellow is amitu or whatever.


----

in the architecture allow for the proxy browsing for device, so this is for when
we want foreign account to not know device ip at all, as even the browsing id52
can disclose ip address during p2p setup, so in that case the request to browse
foreign accounts will be proxied via the device owning account. this introduces
latency in browsing, and for most people not being able to identified is enough,
no absolutely my ip must not be visible at all cost, so we can do the delegated
method. also in delegation we anyways are disclosing ourselves so abs privacy is
not the goal there. so this is only for anonymous browsing mostly.

----

documents will be just json documents. we will have special documents tho, like
account-id52/readme (public readme that the account-id52 owner is maintaining
about this account, it will have some special fields discussed later. then we
will have account-id52/notes, which are my notes about this account-id52. my
notes are only shared with my-account and my devices. similarly we have
device-id52/readme, which are my data about device, synced between account and
all devices, so device alias etc can be seen everywhere. we tend to put things
in automerge documents which auto syncs stuff. so for example now that we have
account-id52/readme,private, we do not necessarily need fastn_account table, as
if it was table the data sync will require custom code, but if it is automerge
our automerge logic will take care of syncing all documents, and if we really
need we can extrat data from such automerge documents and put them in sql tables
for easier querying etc.
