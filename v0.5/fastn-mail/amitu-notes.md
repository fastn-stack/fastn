lets create a type AccountToAccountMessage in fastn-account. this type will be
the messages (we will have other type of message, DeviceToAccount in future when
we have │
│ device entity). For now this would be an enum with only one case for handling
peer to peer email. we are going to deliver a email using this type. lets just
focus on the │
│ type for now, and will plan how to plumb it in networking code or writing the
message handler code to send it on one side and to update local mailbox on the
other side │
│ later.

------------

I am not happy with the mail related types. we are trying to interop with real
world, actual email clients, over IMAP and SMTP. we have to support full SMTP
supported email │
│ as we have no control over what we will receive, we are promising a SMTP
server that just works.



-----------------


so before code lets write a writeup on mail storage, lets first ignore the
current code, as they were written before requirements were really
studied/understood. so we have │
│ to design a mail storage system, we have fastn-account, that stores mails in
the mails folder and in mail.sqlite. now how the mails in folders be stored? how
the sql files │
│ be maintained. then we have to map all SMTP and IMAP operations to those apis.
fastn-mail is created for this purpose, so we are going to have to create
methods etc in │
│ fastn-mail to be called from SMTP/IMAP and also review the requriements in
sent in my previous message lets create a type AccountToAccountMessage in
fastn-account. this type │
│ will be the messages (we will have other type of message, DeviceToAccount in
future when we have │
│ device entity). For now this would be an enum with only one case for handling
peer to peer email. we are going to deliver a email using this type. lets just
focus on the │
│ type for now, and will plan how to plumb it in networking code or writing the
message handler code to send it on one side and to update local mailbox on the
other side │
│ later.


-----------------

need more functions, for email delivery, we will have a every min task which
will ask Mail if there are any id52s who need a mail from us. and if the peer
contacts us, we │
│ will need a function which will give list of emails to deliver to this peer.
the from/to should contain username@id52 parsing to store id52 so these
functions can be │
│ implemented.
