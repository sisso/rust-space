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

## Multiple serilizations

The plan is to have at least 3/4 serialization: Configuration files, save games, FFI, network. Not all are the same, 
but probably will better to have same structure for all. 

## Layers

Better abstraction between storage/indexing/consistency and game logic. For instance, commands hold references and is
responsible to run logic in tick.
