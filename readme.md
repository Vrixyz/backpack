# Backpack

Inventory backend system

## Features

- [ ] authentication via https://www.biscuitsec.org/ ; through third party oauth (github...)
- [ ] Create users via first authentication
- [ ] Create, Read, Update (addition), Delete items (string, int)

## High level usage

- [ ] setup your secrets 
  - [ ] private key for biscuit
  - [ ] oauth third party
  - [ ] database connection
- [ ] start the backpack server
- [ ] use the admin interface to set items
- [ ] To signin/signup, client identify through an oauth third party which callbacks to backpack server to create a new user or identify and get a biscuit token.
- [ ] Clients can use their amount directly by hitting backpack server.
- [ ] To refill, they should hit another server responsible for refilling amount. (through soft or hard currency)

# Thanks

A lot of code inspired by https://github.com/vleue/jornet