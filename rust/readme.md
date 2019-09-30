# TODO

- fix multiples actions per tick
- refactory the everything
    - jump gates be entities
    
## Configuration files

- use hacon to bootstrap and configuration
    
## Save game

- make easy serialization using serde automatic bind    
    
## FFI    

- game api uses plain structures
- game api uses flatbuffers

# Forum

## Multiple serilizations

The plan is to have at least 3/4 serialization: Configuration files, save games, FFI, network. Not all are the same, 
but probably will better to have same structure for all. 
