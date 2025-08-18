so we are not going to do, instead of going to do something novel. first since
we have one account multiple devices, we will create a unique guid like id for
each account, and then we will give each device an id like <account-guid> -1/2
and so on, one to each device. so any edit that happens we will do using this
id, and this id will be available till account, but when account is sharing any
document with any peer, it would have picked a unique local alias to talk to
that peer, so we will rewrite the document history with that < alias-id52>-<
device-serial-number>. so peer will see all edits coming from the alias-id52. we
can even make it a rule that if a document is shared by alias id52, when they
update the document and share back the automerge patch, the history they added
must belong to their alias-id52.


----

│>we will have to store actor guid in the file system this is because automerge
setup will require actor guid, so initial setup maybe circular, not sure. there
is no advantage │
│ of storing this in auto merge because this must never sync, and we should not
lose it, also there is no reason to ever change it

-----

we should pass actor id during Db struct creation. only when getting mergable
data/patch data we should do history rewrite, so those function should get final
alias to use │
│ for history rewrite. also the file should be called automerge.actor-id

-----

lets write a new tutorial, for someone building peer to peer system and is using
this library. assume they need alias feature, so when generating patch we have
to speak │
│ about alias id input. basically describe fastn_documents and tables, and show
what happens on the following operation: 1. document is created (tell me rust
code and sql), 2. │
│ document is updated (how would we keep a list of peers who have access to this
document, and are not out of date, so we should sync with them, and 3. when such
a pair is │
│ themselves sharing changes they have done and we have to update our local.
lets call it P2P-TUTORIAL.md in fastn-automerge crate

-----
in most cases there would be a single alias, and most shares with use same
alias, and cross alias sharing of same document would be rare, but we will pay
the history rewrite │
│ price on every sync. so if we can store the initial alias, and use that for
all future edits to this document, and when asking for share, the alias id we
can check against │
│ the db stored one, and only rewrite if they differe. also we have to store the
group feature and permissions etc in automerge, entire permission system has to
be managed by │
│ this crate. we have to do it such that say for example if some document is
shared readonly with us, we can not make any edits to it at all. when sharing
documents with a │
│ peer we have to also share the peer permission so peer can store them.


----

this function should get two ids, one is self alias, and other is the target
peer id to whom we are sending the patch to. also how would we know what
documents are out of
date, how would we track that they have changes not yet synced with peer, this
would be needed when a peer comes online, so we will probably just ask
fastn-automerge to give
us all patches that i have to share with this peer when that peer comes online.



-----

we need more apis, to create a new group, to add a group to another group, to
add a user to a group. for each group we will have an automerge document
/-/groups/<group-name>. when granting access to document we will need both group
and account related functions.


-----

we have both mine/-/relationships/{alias-id52} and {alias-id52}/-/notes (which
actually should be /-/{alias-id52}/notes (fix this)), do we need both? i think
notes is
enough. also lets note that in notes we store permission that that alias has,
and that decides for example if they can manage groups, otherwise only account
owner or their
devices can manage groups.
