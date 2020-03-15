# Specs

Specs has a very confuse way to produce output. Looks like in Idea mostly of println
in system are never output. 

# TODO

- traders/miners always go to same targets
- fix multiples actions per tick
- switch movements to curves instead interact every frame
- create new movement system. 

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

- change TotalTime and DeltaTime to u64 (analyze the impact of convert into float for math multiplications)
- change Seconds to DeltaTime
- remove reference for simple values
- Create ActionProgressComplete and implement delay actions with ActionProgress
- change cargo to u32

# Modules

## factories
## ship-internal
## high-level-ai
## game

