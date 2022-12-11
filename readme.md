# Backpack

Inventory backend system

![Raw target implementation](Docs/Backpack.drawio.png)

## Features

- [ ] authentication via https://www.biscuitsec.org/ ; through third party oauth (github...)
- [ ] Create users via first authentication
- [ ] Create, Read, Update (addition), Delete items (string, int)

## High level usage

- [x] setup your secrets 
  - [x] private key for biscuit
  - [x] oauth third party
  - [x] database connection
- [x] start the backpack server
- [ ] use the admin interface to set up items and server to server communication
- [ ] To signin/signup, client identify through an oauth third party which callbacks to backpack server to create a new user or identify and get a biscuit token.
- [ ] Clients call to another logic server to refill or use their items, those operations should be done through server-server communication. 
- [ ] this logic server uses data from backpack.

# Thanks

A lot of code inspired by https://github.com/vleue/jornet