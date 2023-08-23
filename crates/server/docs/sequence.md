# First steps

The first step to use backpack is to create a user account.

```mermaid
sequenceDiagram
    user->>+app: I want to create an account
    app->>+backpack: create an account?
    backpack->>+DB: create user
    backpack->>user:  send mail with password
    backpack->>-app: OK
```

Then you can login either as:

- **admin**: to manage apps and their items
- **user**: to authenticate, and modify your items owned as a player from apps you play.

```mermaid
sequenceDiagram
    user->>+app: I want to login with email/password as admin or user
    app->>+backpack: login?
    backpack->>DB: check email/password
    backpack->>-app: credentials
    app->>-app: store credentials
```

Once you have your credentials, keep in mind you'll need to refresh these or your session might be terminated. Check how it's done in the [example game](/crates/example_game_lazy)


# Admin

With any user account, you can login as **admin**, then:

- create apps
- add items to an app you created.
- :construction: later, give rights to foreign apps for an item you manage ; with specific rights (read/write/increase/substract ?)

```mermaid
sequenceDiagram
    user->>app: I want to create an app
    app->>+backpack: create the app?
    backpack->>backpack: check credentials
    backpack->>DB: create the app
    backpack->>-app: OK, appId

    user->>app: I want to create an item for given appId
    app->>+backpack: create the item?
    backpack->>backpack: check credentials
    backpack->>DB: check user manages app
    backpack->>DB: create the item
    backpack->>-app: OK
```

# User

With any user account, you can login as **user**, then use any game using backpack, to modify your items.

## Client-only item modification

The easiest way to modify items is to ask backpack to modify an item from the client app.

:warning: An important rule in software development is to never trust user input.

This solution is considered "insecure", as clients could "easily" cheat by forging requests to ask for unverified modification.

Despite that, it can be useful for:

- fast prototyping, game jams
- non-critical items
  - single player games
  - items not related to balancing
  - items not having a "better" value, such as random seed, population amount in a sandbox city, id for level selection...?
  - ...

```mermaid
sequenceDiagram
    user->>app: I want to modify an item
    app->>+backpack: modify item?
    backpack->>backpack: check credentials (user from an app)
    backpack->>DB: check item can be modified in client-only fashion ("insecurely") by this app
    backpack->>DB: modify the item
    backpack->>-app: OK
```

## App verified item modification

:construction: WIP, this part would need:

- an app password
- an app role
- maybe a password specific app, to pass to each requests, easier for developers to implement than maintaining a bisuit token.

```mermaid
sequenceDiagram
    user->>+app: I'm playing normally, or explicitly want to modify an item
    app->>+app server: play normally, or modify item?
    app->>app server: verify the credentials
    app server->>+backpack: modify item? (+ APP credentials)
    backpack->>backpack: check credentials (app)
    backpack->>+DB: check item can be modified by this app
    backpack->>DB: modify the item
    backpack->>-app server: OK
    app server->>-app: OK
    app->>-user: user sees his items are modified.
```
