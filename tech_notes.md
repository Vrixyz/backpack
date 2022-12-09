As I don't have a strong experience in backend development, 
these changes are mostly instinctive and might not be good practices, 
please reach out if you think I'm writing nonsense :heart:.

# Changes from Jornet

## PostgreSQL

- I avoid UUID in favor of serial primary keys, because I'm not sure about UUID benefits (and plain INT are probably more performant anyway).
- I try to use foreign keys in DB :shrug:

## Auth

- (wip) dropped authentication by id for only oauth.


## Data "renames"

- Admin is now User
  - AdminAccount -> UserId
- 