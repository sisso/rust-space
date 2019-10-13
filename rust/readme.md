# TODO

- fix multiples actions per tick
      
## Configuration files

- use hacon to bootstrap and configuration
    
## Save game

- make easy serialization using serde automatic bind    
    
## FFI    

- game api uses plain structures
- game api uses flatbuffers
- send docked state

## GUI

## refactorying

- change TotalTime and DeltaTime to u64
- change Seconds to DeltaTime
- remove reference for simple values

# Forum

## Actions and commands

### Problem 

We want to have many actions and commands: Command { type = mine }, CommandMine, CommandTrade, etc

Each command should be a component
- filtering and specific systems

Each component should be mutually exclusive
- systems will compete against each other, only one system should be active

Be centralized
- always fetch all possible components to define current one could be very intensive
- for display / status

Each change in actions need to keep the state, so usually write at least into main Component and exclusive. 
- what happens with previous one? will be leak? Or you need to access all containers to be removed?

### Solutions

1. A main system that sanitize current situation by removing not active components. All systems should always join
   with root component to double check that is the active one.
   
2. Nobody should directly act into Command* components, but only make requests that a central CommandSystem will process
   the request by choosing one, update the root, create the specific and remove deprecated elements. 

3. Have a single enum
- this will force every system to always pass through all elements

4. Utilize a single enum as command, but use specific components just as markers for filtering. Kind of 1, but since
   most of data is already in root object, looks less verboes, and less dangeours if we have wrong markers.
- looks less flexible in sitatuations like FFI, serizerliation, etc. Is easy to just add a new number inm classic enum
 for the new component and a new class with new stuff. A single entity means we always need to convert it back.

### Considerations

Each command can be implemented by multiples systems. 

Markers components can be common as optimization if are harmless, just improve best case scenery.

## Multiple serializations

The plan is to have at least 3/4 serialization: Configuration files, save games, FFI, network. Not all are the same, 
but probably will better to have same structure for all. 

## Layers

Better abstraction between storage/indexing/consistency and game logic. For instance, commands hold references and is
responsible to run logic in tick.
