# Forum

## Spreading commands

Both mining and trade have the issue of multiple AI will tend to select the same targets.

To better balance during the high level command where a new target is selected, a reduction score should be applied
for targets that are already destination.

let asteroid_mining = 
    mining_commands.all()
    .map(|i| i.target_id)
    .group_by(|i| (i, 1))
    .map(|(id, vec)| (id, vec.len()))
    .collect();
      

let candidates = 
    search_by_distance[Type](sector_id)
        .take_while(|i| i.distance < 3)
        .map(|i| (i.id, i.distance + asteroid_mining.get(i.id).get_or_else(0))
        .collect()
        .sort()
        .into_iter()
        .next()



## Generic search by position

search_by_distance[Asteroid](sector_id)

## FFI events

We should add new entities independent of location

New events 
DockedAt(target), Undock(sector, pos), Teleport(sector,pos)

### Context

Currently create entity add a EntityAdded with position and sector. 
- can not be used for created docked entity

The idea was to Add when visible and Remove when invisible. 
- that will complicate drastically UI, like 



## AtSectorId

One more time come to the discussion if we should have Location::AtSector or a diffirecent component. While
centralize most of position algorithms become much simplier, some systems that dont care about position like
is not good to have a position.

- Neh, only because you are not doing right and searching not only for the nearest secort, but nearest 
  position. NEGATE!!!


## Factory and Shipyard and input ores

Both require inputs, both potentially require cargo reservation, both require cargo delivering. 

Cargo deliver will be always managed by cargo to have consistency through whitelist. 

### Options

a) Check for existence of factory, shipyard or other component to decide?
- simple for now until have a better decision
b) Move everything to factory, and shipyard consumes directly from factory?
- Still shipyard will consume a Ware, this ware need to be in cargo.
  - can other ships take this cargo?
  - how I generate cargo demand back?
c) Convert shipyard into a factory?
- Instead a ware output is a ship_id? 
  - look simple to add, but reduce the cohesion
  - what if we have a new requirement later?
d) Create a new component that contains a InputRequest?
- factory/station ai need to keep it in sync 
- maybe is the born a big management AI thorugh requests and demands. Instead give commands, it can schedule tasks and
  only schedule new InputRequest giving priority, until have full cargo, etc
- Maybe will be the way to user see and create new commands.
c) Check cargo whitelist?
- and how it will distinguish between input and output?

Conclusion

a) is easy solution for now
d) looks the most flexible and one that open broad flexibility. Still need to confirm if spreading the AI decision is 
   not a bad idea. (review code of prototype high_level_ai)

## Factory and cargo spaces

Factories need to be able to control the amount of space used by its cargo, otherwise a single input can force
enter in dead lock to receive a secondary input or release the output.

AI is not enough, will be very prone to failure and users will not obey.

Cargo whitelist % is the best approach to have consistency in the cargo space. Whitelisting a % of total cargo, or using
multiple storages.
- how factory component will manage cargo storage?
    - the way a component can manage other is by creating a system, a system that take every factory and cargo pair and
      setup its storage reserve. A storage or extra component will be require to cache already processed factories.
    - during object instantiation the cargo is updated given other components, if any system changes one of requirements,
      it will need to manually setup the cargo again

## Sector as Component

Sector and Jump become a component

Pathfind will just list Sectors or index.

SectorId become the entity.

## StaticId

- Some objects will require to have a global ID. Like Sector, WareId or prefab.
- Most of the time they will be related with references of bootstrap configuration.
- We can use a component. 
  - how to deal with indexing if some use cases will require read after set?

## New Movement system

Instead have just a velocity or acceleration. The movement will be always defined by a curve with start position, 
start time, and t0. A complete navigation will be a sequence of those curves. 

This not only will facilitate integration with UI and allow different cycle time. But simplify code, allow prediction
for pathfinding, allow easy use of bezier, visualize, etc.

## FFI and Events

FFI need to notify any changes to the controller. 

It should notify only relevant information. We can implement on demand from GUI requirements.

FFI information need to transfer very dry, no complex.

We want to move all FFI related stuff to a new module. This new module will have a system (how to create system into 
other modules???) that will convert the events change into raw FFI data.

All changes will generate Event.

### Solution A: Attach a event to the entity

FFISystem will process all Event and populate FFI messages queue. 

How to support multiple events for same entity? 
- you can't, so this solution does not works

### Solution B: Create new entity to hold the vent

Alternative we can create a new entity per event message. Use LazyUpdate to insert events 
in parallel. Entity creatino overhead can be reduced by holding list.

Looks very ECS way of doing it.

Manage component registering using LazyUpdate is error prone

### Solution C:

Use a single Event resource with multiple producer single reader.

## ActionPpogress

Action progress is a simple way to standardize delays. Instead always have to get the current
action and check against total time. The system can just check if no delay is active.

A delay system can centralize all delay logic.

An issue is that all actions will require to be executed synchronous for all actions that
can activate delay, since they will require write capability. Anyways, is already what is happening by using 
ActionActive

Mostly of action control is already done by the Handler. Late update is a option to all
progress creation.

How the waiting action will know that its action is complete?
- a new ActionProgressComplete component need to be inserted

So all actions will only start if have no ActionProgress
Timed actions will only complete when a ActionProgressCompleted exists

## Location

A single enum with multual exclusive is much easy to manipulate. While in many cases a single component AtSectorId make
sense, in the end of the they you will always require to access other components like docked or position.

With 3 components (Pos, Sector, DocketAt) is too verbose to implement single methods like

is_near(pos_repo, sector_repo, docked_repo, entty_A, enitty_b) -> bool

Operations like only non docked or any with sector are easy to implement in the start, but in th end of day will always
need to be implemented by some pathfinding compatible index.`

## Split commands between systems

The forces to split a algorithm into different systems are:
- allow parallelism (when they are not dependent)
- reduce amount of dependencies
- facilitate automatic test

While the initially looks promising. In reality:
- split a algorithm between systems usually create dependencies that can not be parallelize
- reduce dependencies and facilitate test can be done easy by simple delegating code specific methods, create and
  maintain systems are still scale level of complexity tha simple methods.
  
Conclusion:

Complexity should be sliced by methods, not systems. Only split systems for tasks that are complete independent.

## Mine

Command mine each second will extract X amount of ore from resource

Resource should respawn

CommandMine
CommandMineTarget
NavigationRequest
Navigation
ActionMove
ActionMine

### Current 

When => Do

CommandMine && !CommandMineTarget && !CommandMineDeliver => 
    if cargo is full {
        find deliver station and add CommandMineDeliver
    } else {
        find mine target and add CommandMineTarget
    }
    
CommandMine && CommandMineTarget && !Navigation && !ActionMine => 

    if cargo is full {
        remove CommandMineTarget        
    } else 
    if target is near {
     add ActionMine
    } else {
     add NavigationRequest
    }
   
ActionMine => 
    if cargo full || target is far {
        remove ActionMine
    }
    
### Simplify

CommandMine { state: enum MineState { Mine { target_id }, Deliver { target_id } }

CommandMine && !Nav =>
    if cargo is full {
        remove ActionMine if exists
        if MineState::Deliver { target_id } {
            if near {
                dock
            }
            else {
                add nav_request to target_id
            }
        } else {
        }
    }
    
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

Execute one command or action is a real requirement?

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

## Location

While have 2 distinct allow better filtering and easy mainipulation in most of situations we know that things are in
space. Its create a lot of boilerplate to remember to double check if entity is docked.

- Should not be a problem, if you are filtering by space, you dont expected to be docked. If you want both you 
  manually should query both components

- Anyway Docked is never easy to use since require a new lookup into current position