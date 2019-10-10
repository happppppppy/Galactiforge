-- Importing lovetoys
local lovetoys = require "code/lib/lovetoys"
lovetoys.initialize({
	globals = true, 
	debug = true
})

-- Importing helper functions
local helper_functions = require("code/src/helper_functions")

global_zoom_level = 1
factory_id = 0

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

--Event components

--Property components
require("code/src/components/properties/Health")
require("code/src/components/properties/Faction")
local Health, Faction = Component.load({"Health", "Faction"})

--Object components
require("code/src/components/objects/GridMaster")
local GridMaster  = Component.load({"GridMaster"})

--Ammunition components
require("code/src/components/objects/ammunition/Bullet")
local Bullet = Component.load({"Bullet"})

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
require("code/src/components/objects/grids/Factory")
require("code/src/components/objects/grids/Weapon")
require("code/src/components/objects/grids/Thruster")
local Factory, Weapon, Thruster, GridItem, GridInventory, GridTransfer, GridProcessor, GridBaseGraphic, GridHeat = Component.load({"Factory", "Weapon", "Thruster", "GridItem", "GridInventory", "GridTransfer", "GridProcessor", "GridBaseGraphic", "GridHeat"})

--Grid systems
WeaponSystem = require("code/src/systems/grids/WeaponSystem")
ThrusterSystem = require("code/src/systems/grids/ThrusterSystem")
GridPhysicsSystem = require("code/src/systems/grids/GridPhysicsSystem")
GridItemSystem = require("code/src/systems/grids/GridItemSystem")
GridTransferSystem = require("code/src/systems/grids/GridTransferSystem")
GridProcessorSystem = require("code/src/systems/grids/GridProcessorSystem")
GridMasterSystem = require("code/src/systems/grids/GridMasterSystem")
GridHeatSystem = require("code/src/systems/grids/GridHeatSystem")

--Graphic systems
RenderHUDSystem = require("code/src/systems/graphic/RenderHUDSystem")
RenderSpriteSystem = require("code/src/systems/graphic/RenderSpriteSystem")
RenderGridMasterSystem = require("code/src/systems/graphic/RenderGridMasterSystem")
RenderParticleSystem = require("code/src/systems/graphic/RenderParticleSystem")
UpdateParticleSystem = require("code/src/systems/graphic/UpdateParticleSystem")
TileSetGridAnimatorSystem = require("code/src/systems/graphic/TileSetGridAnimatorSystem")


--Physics systems
SetupPhysicsSystem = require("code/src/systems/physics/SetupPhysicsSystem")
CleanupSystem = require("code/src/systems/physics/CleanupSystem")

--Event systems
MainMouseSystem = require("code/src/systems/event/MainMouseSystem")
MainKeySystem = require("code/src/systems/event/MainKeySystem")
DamageSystem = require("code/src/systems/event/DamageSystem")
BulletSystem = require("code/src/systems/event/BulletSystem")
FieryDeathSystem = require("code/src/systems/event/FieryDeathSystem")

--Controller systems
AIControllerSystem = require("code/src/systems/controllers/AIControllerSystem")
PlayerControllerSystem = require("code/src/systems/controllers/PlayerControllerSystem")

--Events
require("code/src/events/KeyPressed")
require("code/src/events/MousePressed")
require("code/src/events/DamageOccured")
require("code/src/events/GridSelected")
require("code/src/events/FireEvent")
require("code/src/events/ThrusterEvent")

function love.load()

	--Game data
	datasets = {}
	files = love.filesystem.getDirectoryItems( "code/src/configs/" )
	for i,v in pairs(files) do
		local category = v:match("(.+)%..+")
		local path = "code/src/configs/"..category..".json"
		local data = helper_functions.openjson(path)
		for i,v in pairs(data) do
			v.category = category
			datasets[i] = v
		end
	end

	global_component_name_list = {}
	for i,v in pairs(datasets) do
		if v.category == "armor" or v.category == "factory"or v.category == "technology" or v.category == "thruster" or v.category == "weapon" then 
			global_component_name_list[#global_component_name_list + 1] = i
		end
	end

	ship_data = helper_functions.openjson("code/src/configs/ships.json") 

	--Tilesets
	tileset_small = helper_functions.createTileset("assets/images/galactiforge_tilesets/factory_devices.png", 32, 32)

	--Fonts
	font_hud = love.graphics.newFont("assets/fonts/GiantRobotArmy-Medium.ttf")

	--Global variables
	global_show_resource_count = false
	lock_view_to_ship = false
	global_component_index = 1
	global_component_directions = {0,90,180,270}
	global_component_direction_index = 1
	global_target_list = {}
	global_ai_active = true
	global_build_mode = false

	--Canvases
	backgroundImage = love.graphics.newImage("assets/images/space_breaker_asset/Background/stars_texture.png")
	backgroundQuad = love.graphics.newQuad(0,0,backgroundImage:getWidth()*1.1,backgroundImage:getHeight()*1.1,backgroundImage:getWidth(),backgroundImage:getWidth())
	backgroundImage:setWrap("repeat", "repeat")
	
	love.physics.setMeter(32) --the height of a meter our worlds will be 32px
	world = love.physics.newWorld(0, 0, true) --create a world for the bodies to exist in with horizontal gravity of 0 and vertical gravity of 0

	engine = Engine()

	eventmanager = EventManager()

	eventmanager:addListener("KeyPressed", MainKeySystem(), MainKeySystem().fireEvent)
	eventmanager:addListener("MousePressed", MainMouseSystem(), MainMouseSystem().fireEvent )
	eventmanager:addListener("DamageOccured", DamageSystem(), DamageSystem().fireEvent)
	eventmanager:addListener("GridSelected", GridMasterSystem(), GridMasterSystem().fireEvent)
	eventmanager:addListener("FireEvent", WeaponSystem(), WeaponSystem().fireCannon)
	eventmanager:addListener("ThrusterEvent", ThrusterSystem(), ThrusterSystem().fireEvent)

	engine:addSystem(RenderParticleSystem())
	engine:addSystem(RenderSpriteSystem())
	engine:addSystem(BulletSystem())
	engine:addSystem(SetupPhysicsSystem())
	engine:addSystem(RenderGridMasterSystem())
	engine:addSystem(GridMasterSystem())
	engine:addSystem(GridPhysicsSystem())
	engine:addSystem(GridItemSystem())
	engine:addSystem(RenderHUDSystem())
	engine:addSystem(DamageSystem())
	engine:addSystem(FieryDeathSystem())
	engine:addSystem(UpdateParticleSystem())
	engine:addSystem(WeaponSystem())
	engine:addSystem(GridHeatSystem())
	engine:addSystem(ThrusterSystem())
	engine:addSystem(TileSetGridAnimatorSystem())
	engine:addSystem(CleanupSystem())

	engine:addSystem(AIControllerSystem())
	engine:addSystem(PlayerControllerSystem())

	engine:addSystem(GridProcessorSystem())
	engine:addSystem(GridTransferSystem())

	-- playerShip_type = "intruder"
	-- playerShip = Entity()
	-- playerShip:add(GridMaster(ship_data[playerShip_type].starter_grid, ship_data[playerShip_type], 0.5, 32, 32, 1, 1))
	-- playerShip:add(PositionPhysics(world,500,600,math.rad(180),"dynamic"))
	-- playerShip:add(PlayerController())
	-- playerShip:add(Health(100))
	-- playerShip:add(Faction("Terran"))
	-- engine:addEntity(playerShip)

	playerShip_type = "micro_bandit"
	playerShip = Entity()
	playerShip:add(GridMaster(ship_data[playerShip_type].starter_grid, ship_data[playerShip_type], 0.5, 32, 32, 1, 1, true))
	playerShip:add(PositionPhysics(world,500,600,math.rad(180),"dynamic"))
	playerShip:add(PlayerController())
	playerShip:add(Health(100))
	playerShip:add(Faction("Terran"))
	engine:addEntity(playerShip)

	-- playerShip_type = "blank_canvas"
	-- playerShip = Entity()
	-- playerShip:add(GridMaster(ship_data[playerShip_type].starter_grid, ship_data[playerShip_type], 0.5, 32, 32, 1, 1))
	-- playerShip:add(PositionPhysics(world,500,600,math.rad(180),"dynamic"))
	-- playerShip:add(PlayerController())
	-- playerShip:add(Health(100))
	-- playerShip:add(Faction("Terran"))
	-- engine:addEntity(playerShip)



	-- opponentShip_type = "blank_canvas"
	-- opponentShip = Entity()
	-- opponentShip:add(GridMaster(ship_data[opponentShip_type].starter_grid, ship_data[opponentShip_type], 0.5, 32, 32, 2, 2))
	-- opponentShip:add(PositionPhysics(world,2000,1900,0,"dynamic"))
	-- -- opponentShip:add(AIController())
	-- opponentShip:add(Health(100))
	-- opponentShip:add(Faction("Splorg"))
	-- engine:addEntity(opponentShip)

		opponentShip_type = "micro_bandit"
		opponentShip = Entity()
		opponentShip:add(GridMaster(ship_data[opponentShip_type].starter_grid, ship_data[opponentShip_type], 0.5, 32, 32, 2, 2))
		opponentShip:add(PositionPhysics(world,2000,2000,0,"dynamic"))
		opponentShip:add(AIController())
		opponentShip:add(Health(100))
		opponentShip:add(Faction("Splorg"))
		engine:addEntity(opponentShip)

		
		opponentShip_type = "micro_bandit"
		opponentShip = Entity()
		opponentShip:add(GridMaster(ship_data[opponentShip_type].starter_grid, ship_data[opponentShip_type], 0.5, 32, 32, 2, 2))
		opponentShip:add(PositionPhysics(world,-4000,-3000,0,"dynamic"))
		opponentShip:add(AIController())
		opponentShip:add(Health(100))
		opponentShip:add(Faction("Splorg"))
		engine:addEntity(opponentShip)


	lines = {}
end

local elapsedTime = 0

function love.update(dt)

		world:update(dt)
		world:setCallbacks(beginContact, endContact, preSolve, postSolve)
		engine:update(dt)

end

function love.draw()
	width = love.graphics.getWidth()
	height = love.graphics.getHeight()

	playerShipLoc = playerShip:get("PositionPhysics")
	playerShipSprite = playerShip:get("Sprite")

	love.graphics.draw(backgroundImage, backgroundQuad, 0, 0)
	love.graphics.scale(global_zoom_level)

	if global_build_mode then

		local x_loc = playerShipLoc.body:getX()*-1+width*(1/global_zoom_level)/2
		local y_loc = playerShipLoc.body:getY()*-1+height*(1/global_zoom_level)/2
		
		love.graphics.translate(width*(1/global_zoom_level)/2, height*(1/global_zoom_level)/2)
		love.graphics.rotate(playerShipLoc.body:getAngle()*-1)
		love.graphics.translate(-width*(1/global_zoom_level)/2, -height*(1/global_zoom_level)/2)
		love.graphics.translate(x_loc,y_loc)

	elseif lock_view_to_ship then

			local x_loc = playerShipLoc.body:getX()*-1+width*(1/global_zoom_level)/2
			local y_loc = playerShipLoc.body:getY()*-1+height*(1/global_zoom_level)/2
	
			love.graphics.translate(x_loc,y_loc)
	end

	engine:draw()

	for _,v in pairs(lines) do
		love.graphics.line(0,0,v[1],v[2])
	end
end

function love.keypressed(key, isrepeat)
	eventmanager:fireEvent(KeyPressed(key, isrepeat))
end

function love.mousepressed(x, y, button)
	eventmanager:fireEvent(MousePressed(x, y, button))
end

function love.wheelmoved( dx, dy )
	global_zoom_level = global_zoom_level + dy * 0.1
end

function beginContact(a, b, coll)
	eventmanager:fireEvent(DamageOccured(a:getUserData(), b:getUserData()))
end

function endContact(a, b, coll)
end

function preSolve(a, b, coll)
end

function postSolve(a, b, coll, normalimpulse, tangentimpulse)
end