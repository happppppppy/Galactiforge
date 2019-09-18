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
require("code/src/components/objects/grids/Factory")
require("code/src/components/objects/grids/Weapon")
require("code/src/components/objects/grids/Thruster")
local Factory, Weapon, Thruster, GridItem = Component.load({"Factory", "Weapon", "Thruster", "GridItem"})

--Grid systems
WeaponSystem = require("code/src/systems/grids/WeaponSystem")
ThrusterSystem = require("code/src/systems/grids/ThrusterSystem")
GridPhysicsSystem = require("code/src/systems/grids/GridPhysicsSystem")
GridItemSystem = require("code/src/systems/grids/GridItemSystem")

--Controller systems
GridMasterSystem = require("code/src/systems/controllers/GridMasterSystem")

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

--Events
require("code/src/events/KeyPressed")
require("code/src/events/MousePressed")
require("code/src/events/DamageOccured")
require("code/src/events/GridSelected")


function love.load()

	--Game data
	datasets = {}
	factory_data = helper_functions.openjson("code/src/configs/factories.json")
	for i,v in pairs(factory_data) do
		v.category = "factory"
		datasets[i] = v
	end
	weapon_data = helper_functions.openjson("code/src/configs/weapons.json")
	for i,v in pairs(weapon_data) do
		v.category = "weapon"
		datasets[i] = v
	end
	thruster_data = helper_functions.openjson("code/src/configs/thrusters.json")
	for i,v in pairs(thruster_data) do
		v.category = "thruster"
		datasets[i] = v
	end

	global_component_name_list = {}
	for i,v in pairs(datasets) do
		global_component_name_list[#global_component_name_list + 1] = i
	end

	ship_data = helper_functions.openjson("code/src/configs/ships.json") 


	--Tilesets
	tileset_small = helper_functions.createTileset("assets/images/galactiforge_tilesets/factory_devices.png", 32, 32)

	--Fonts
	font_hud = love.graphics.newFont("assets/fonts/GiantRobotArmy-Medium.ttf")

	--Global variables
	global_show_resource_count = true
	lock_view_to_ship = true
	global_component_index = 1
	global_component_directions = {0,90,180,270}
	global_component_direction_index = 1
	--Canvases
	canvas_hud = love.graphics.newCanvas()

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
	engine:addSystem(ThrusterSystem())
	engine:addSystem(TileSetGridAnimatorSystem())
	engine:addSystem(CleanupSystem())
	local station_grid = {
		{["category"]="factory", ["type"]="ore_hopper",["x"]=-1,["y"]=-1},
	{["category"]="factory", ["type"]="ore_crusher",["x"]=0,["y"]=-1},
	{["category"]="factory", ["type"]="ore_separator",["x"]=1,["y"]=-1},
	{["category"]="factory", ["type"]="arc_furnace",["x"]=1,["y"]=0},
	{["category"]="factory", ["type"]="casting_line",["x"]=1,["y"]=1}, 
	{["category"]="factory", ["type"]="machining_centre",["x"]=0,["y"]=1}, 
	{["category"]="factory", ["type"]="assembly_centre",["x"]=-1,["y"]=1},
	{["category"]="weapon", ["type"]="cannon", ["x"]=-1, ["y"]=0}}
	local grid = {
		{["category"]="factory", ["type"]="ore_hopper",["x"]=-1,["y"]=1},
		{["category"]="factory", ["type"]="ore_crusher",["x"]=-1,["y"]=0},
		{["category"]="factory", ["type"]="ore_separator",["x"]=-1,["y"]=-1},
		{["category"]="factory", ["type"]="arc_furnace",["x"]=-1,["y"]=-2},
		{["category"]="factory", ["type"]="casting_line",["x"]=-1,["y"]=-3}, 
		{["category"]="factory", ["type"]="machining_centre",["x"]=-1,["y"]=-4}, 
		{["category"]="factory", ["type"]="assembly_centre",["x"]=-1,["y"]=-5},
		{["category"]="factory", ["type"]="ore_hopper",["x"]=1,["y"]=1},
		{["category"]="factory", ["type"]="ore_crusher",["x"]=1,["y"]=0},
		{["category"]="factory", ["type"]="ore_separator",["x"]=1,["y"]=-1},
		{["category"]="factory", ["type"]="arc_furnace",["x"]=1,["y"]=-2},
		{["category"]="factory", ["type"]="casting_line",["x"]=1,["y"]=-3}, 
		{["category"]="factory", ["type"]="machining_centre",["x"]=1,["y"]=-4}, 
		{["category"]="factory", ["type"]="assembly_centre",["x"]=1,["y"]=-5},
		{["category"]="weapon", ["type"]="cannon", ["x"]=1, ["y"]=-6},
		{["category"]="weapon", ["type"]="cannon", ["x"]=2, ["y"]=-5},
		{["category"]="weapon", ["type"]="cannon", ["x"]=-1, ["y"]=-6},
		{["category"]="weapon", ["type"]="cannon", ["x"]=-2, ["y"]=-5},
		{["category"]="thruster", ["type"]="large_thruster", ["x"]=-1, ["y"]=7, ["direction"] = 0},
		{["category"]="thruster", ["type"]="large_thruster", ["x"]=1, ["y"]=7, ["direction"] = 0},
		{["category"]="thruster", ["type"]="mini_thruster", ["x"]=-2, ["y"]=6, ["direction"] = 90},
		{["category"]="thruster", ["type"]="mini_thruster", ["x"]=2, ["y"]=6, ["direction"] = 270},
		{["category"]="thruster", ["type"]="mini_thruster", ["x"]=-2, ["y"]=5, ["direction"] = 90},
		{["category"]="thruster", ["type"]="mini_thruster", ["x"]=2, ["y"]=5, ["direction"] = 270},
		{["category"]="thruster", ["type"]="large_thruster", ["x"]=-1, ["y"]=-7, ["direction"] = 180},
		{["category"]="thruster", ["type"]="large_thruster", ["x"]=1, ["y"]=-7, ["direction"] = 180},		
		{["category"]="factory", ["type"]="fuel_generator",["x"]=1,["y"]=6}, 
		{["category"]="factory", ["type"]="fuel_generator",["x"]=-1,["y"]=6}, 
		{["category"]="factory", ["type"]="fuel_generator",["x"]=1,["y"]=5},
		{["category"]="factory", ["type"]="fuel_generator",["x"]=-1,["y"]=5},
		{["category"]="factory", ["type"]="fuel_generator",["x"]=0,["y"]=5}
	}

	playerShip_type = "intruder"
	playerShip = Entity()
	playerShip:add(GridMaster(grid, ship_data[playerShip_type], 0.5, 32, 32, 1, 1))
	playerShip:add(PositionPhysics(world,500,600,math.rad(180),"dynamic"))
	playerShip:add(Sprite("assets/images/galactiforge_tilesets/body_01.png", 0, 1, 1, 0, 0, true))
	playerShip:add(PlayerController())
	playerShip:add(Health(100))
	playerShip:add(Faction("Terran"))
	engine:addEntity(playerShip)


	opponentShip = Entity()
	opponentShip:add(GridMaster(grid, ship_data["intruder"], 0.5, 32, 32, 2, 2))
	opponentShip:add(PositionPhysics(world,800,900,0.7,"dynamic"))
	opponentShip:add(Sprite("assets/images/galactiforge_tilesets/body_01.png", 0, 1, 1, 0, 0, true))
	opponentShip:add(AIController())
	opponentShip:add(Health(100))
	opponentShip:add(Faction("Splorg"))
	engine:addEntity(opponentShip)

	destructable = Entity()
	destructable:add(Health(10))
	destructable:add(PositionPhysics(world,200,200,0.1,"static"))
	destructable:add(GridMaster(station_grid, ship_data["station"], 0.5, 32, 32, 3, 3))
	destructable:add(AIController())
	destructable:add(CollisionPhysics("Rectangle",  60, 60, 1, 3, 3))
	destructable:add(Sprite("assets/images/space_breaker_asset/Others/Stations/station.png", 0, 1, 1, 0, 0, true))
	destructable:add(FieryDeath())
	destructable:add(Faction("Splorg"))
	engine:addEntity(destructable)
	
	destructable = Entity()
	destructable:add(Health(10))
	destructable:add(GridMaster(station_grid, ship_data["station"], 0.5, 32, 32, 3, 3))
	destructable:add(AIController())
	destructable:add(PositionPhysics(world,400,400,0.1,"static"))
	destructable:add(CollisionPhysics("Rectangle",  60, 60, 1, 3, 3))
	destructable:add(Sprite("assets/images/space_breaker_asset/Others/Stations/station.png", 0, 1, 1, 0, 0, true))
	destructable:add(FieryDeath())
	destructable:add(Faction("Splorg"))
	engine:addEntity(destructable)
	
	destructable = Entity()
	destructable:add(Health(10))
	destructable:add(GridMaster(station_grid, ship_data["station"], 0.5, 32, 32, 3, 3))
	destructable:add(AIController())
	destructable:add(PositionPhysics(world,600,200,0.1,"static"))
	destructable:add(CollisionPhysics("Rectangle",  60, 60, 1, 3, 3))
	destructable:add(Sprite("assets/images/space_breaker_asset/Others/Stations/station.png", 0, 1, 1, 0, 0, true))
	destructable:add(FieryDeath())
	destructable:add(Faction("Splorg"))
	engine:addEntity(destructable)


	circs = {}
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

	if lock_view_to_ship then
		local x_loc = playerShipLoc.body:getX()*-1+width*(1/global_zoom_level)/2
		local y_loc = playerShipLoc.body:getY()*-1+height*(1/global_zoom_level)/2
		love.graphics.translate(x_loc,y_loc)
	end

	engine:draw()

	for i,v in pairs(circs) do
		love.graphics.circle("fill", v.x, v.y, 2)
	end

end

function love.keypressed(key, isrepeat)
	eventmanager:fireEvent(KeyPressed(key, isrepeat))
end

function love.mousepressed(x, y, button)
	eventmanager:fireEvent(MousePressed(x, y, button))
end

function beginContact(a, b, coll)
	eventmanager:fireEvent(DamageOccured(a:getUserData(), b:getUserData()))
end

function endContact(a, b, coll)
	-- eventmanager:fireEvent(DamageOccured(a:getUserData(), b:getUserData()))
end

function preSolve(a, b, coll)
end

function postSolve(a, b, coll, normalimpulse, tangentimpulse)
end