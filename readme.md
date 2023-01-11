# Backpack

Inventory backend system for **items from different games**.

# Pitch

I have a vision featuring multiple games impacting each others.

Games can be social beyond their limited scope:
> My mom did beat the 6th level of her favorite match3 game, she was able to send me a skin for my favorite FPS!

> My Friend hatched eggs in their clicker game, so I can spawn more dragons in my RTS.

Single players can find their value too:
> I finished my exploration game and gained a garden to use in that other construction game!

> I'm going to play this god-simulation game to add a few planets to my other galaxy-simulation game.

# Getting started

## Understand Backpack
Helpful documentations for the project:

- [sequence diagram](crates/backpack-server/docs/sequence.md): to understand the communication flow
- [database class diagram](crates/backpack-server/docs/database.md): to understand stored data
- [OpenAPI](crates/backpack-server/docs/openapi/openapi3_0.yaml): to start consuming the API
- *[Tech scribbles](Docs/Backpack.drawio.png): first diagram of the project, a less professional-looking sequence diagram.*


## :construction: Use Backpack :construction:

<details>
<summary>From source ?</summary>

- setup your secrets 
  - private key for biscuit
  - oauth third party
  - database connection
- start the backpack server

</details>

- :construction: Get the url of the Backpack server you connect to. Official server wip.
- Use the admin interface to set up items and server to server communication
  - Sign up/Sign in with provided means (email/password or third party (github...))
  - Create an app
  - Create one or several item
  - You will use app's and items' IDs to develop your app which interact with those app/items.
- Choose a strategy to update your items, *see [sequence diagram](crates/backpack-server/docs/sequence.md) for more details*
  - **Lazy:** Clients call directly to Backpack, potentially enabling users to abuse forging requests.
  - **Secure:** Clients call to another logic server *(which you are responsible of)* to refill or use their items, those operations are done through server-server communication.

# Thanks

A lot of code originally inspired by https://github.com/vleue/jornet