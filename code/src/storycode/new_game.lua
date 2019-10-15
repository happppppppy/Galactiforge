-- Controller components
require("code/src/components/controllers/PlayerController")
require("code/src/components/controllers/AIController")
local PlayerController, AIController  = Component.load({"PlayerController", "AIController"})
--Grid components
require("code/src/components/objects/grids/GridItem")
require("code/src/components/objects/grids/GridInventory")
require("code/src/components/objects/grids/GridTransfer")
require("code/src/components/objects/grids/GridProcessor")
require("code/src/components/objects/grids/GridBaseGraphic")
require("code/src/components/objects/grids/GridHeat")
require("code/src/components/objects/grids/Weapon")
require("code/src/components/objects/grids/Thruster")
local Weapon, Thruster, GridItem, GridInventory, GridTransfer, GridProcessor, GridBaseGraphic, GridHeat = Component.load({"Weapon", "Thruster", "GridItem", "GridInventory", "GridTransfer", "GridProcessor", "GridBaseGraphic", "GridHeat"})
--Property components
require("code/src/components/properties/Health")
require("code/src/components/properties/Faction")
local Health, Faction = Component.load({"Health", "Faction"})

--Object components
require("code/src/components/objects/GridMaster")
local GridMaster  = Component.load({"GridMaster"})

--Graphic components
require("code/src/components/graphic/Sprite")
require("code/src/components/graphic/TileSet")
require("code/src/components/graphic/FieryDeath")
require("code/src/components/graphic/Particles")
require("code/src/components/graphic/TileSetGrid")
local Sprite, TileSet, FieryDeath, Particles, TileSetGrid = Component.load({"Sprite", "TileSet", "FieryDeath", "Particles", "TileSetGrid"})

--Physics components
require("code/src/components/physics/PositionPhysics")
require("code/src/components/physics/DynamicPhysics")
require("code/src/components/physics/CollisionPhysics")
local GridPhysics = require("code/src/components/physics/GridPhysics")
local PositionPhysics, DynamicPhysics, CollisionPhysics = Component.load({"PositionPhysics", "DynamicPhysics", "CollisionPhysics"})

local newgame = {}

function newgame.create_ships() 
	playerShip = Entity()
	playerShip:add(GridMaster(ship_data["micro_bandit"].starter_grid, ship_data["micro_bandit"], 0.5, 32, 32, 1, 1, true))
	playerShip:add(PositionPhysics(world,500,600,math.rad(180),"dynamic"))
	playerShip:add(PlayerController())
	playerShip:add(Health(100))
	playerShip:add(Faction("Terran"))
	engine:addEntity(playerShip)

	opponentShip = Entity()
	opponentShip:add(GridMaster(ship_data["micro_bandit"].starter_grid, ship_data["micro_bandit"], 0.5, 32, 32, 2, 2))
	opponentShip:add(PositionPhysics(world,2000,2000,0,"dynamic"))
	opponentShip:add(AIController())
	opponentShip:add(Health(100))
	opponentShip:add(Faction("Splorg"))
	engine:addEntity(opponentShip)

	opponentShip = Entity()
	opponentShip:add(GridMaster(ship_data["micro_bandit"].starter_grid, ship_data["micro_bandit"], 0.5, 32, 32, 2, 2))
	opponentShip:add(PositionPhysics(world,-4000,-3000,0,"dynamic"))
	opponentShip:add(AIController())
	opponentShip:add(Health(100))
	opponentShip:add(Faction("Splorg"))
	engine:addEntity(opponentShip)

end

function newgame.create_environment() 

end

return newgame