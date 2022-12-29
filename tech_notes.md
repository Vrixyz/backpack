As I don't have a strong experience in backend development, 
these changes are mostly instinctive and might not be good practices, 
please reach out if you think I'm writing nonsense :heart:.

# Changes from Jornet

## PostgreSQL

- I avoid UUID in favor of serial primary keys, because I'm not sure about UUID benefits (and plain INT are probably more performant anyway).
- I try to use foreign keys in DB :shrug:

## Auth

- dropped authentication by id for only oauth, added a fake auth to ease testing.
- relax rules of BiscuitInfo to allow adding multiple facts for a given data structure
- use TryFrom rather that BiscuitFact
- https://github.com/maidsafe-archive/system_uri could help with redirect from browser to app.
  - osx functions used are deprecated, I guess https://github.com/burtonageo/cargo-bundle could help declaring url schemes. :warning: use it from git https://github.com/burtonageo/cargo-bundle/issues/125

## Data "renames"

- Admin is now User
  - AdminAccount -> UserId
- 