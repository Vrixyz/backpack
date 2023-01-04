# Admin

The first step to use backpack is to create a user account.

With any user account, you can:
- login as admin
- create an app
- add items to an app you created.

```mermaid
sequenceDiagram
    user->>+app: I want to create an account
    app->>+backpack: create an account?
    backpack->>+DB: create user
    backpack->>user:  send mail with password
    backpack->>-app: OK

    user->>+app: I want to login with email/password as admin
    app->>+backpack: login?
    backpack->>DB: check user/password
    backpack->>-app: credentials
    app->>app: store credentials
    app->>-user: credentials

    user->>app: I want to create an app
    app->>+backpack: create the app?
    backpack->>backpack: check credentials
    backpack->>DB: create the app
    backpack->>-app: OK

    user->>app: I want to create an item for given appId
    app->>+backpack: create the item?
    backpack->>backpack: check credentials
    backpack->>DB: check user manages app
    backpack->>DB: create the app
    backpack->>-app: OK
```

# User

The second step to using backpack is to use items through apps.

## TODO
- login as a user
- use an item (insecurely)
- use an item (securely through a server)
  - it would need to create an "app" password, used by the trusted game server.