-- Importing helper functions
local helper_functions = require("code/src/helper_functions")
local new_game = require("code/src/storycode/new_game")
local objective_generator = require("code/src/storycode/objective_generator")

global_zoom_level = 1

-- This function will return a string filetree of all files
-- in the folder and files in all subfolders

function recursiveEnumerate(folder, fileTree)
	local lfs = love.filesystem
	local filesTable = lfs.getDirectoryItems(folder)
	for i,v in ipairs(filesTable) do
		local file = folder.."/"..v
		local fileinfo = lfs.getInfo(file)
		if fileinfo.type == "file" then
			table.insert(fileTree, file)
		elseif fileinfo.type == "directory" then
			fileTree = recursiveEnumerate(file, fileTree)
		end
	end
	return fileTree
end


-- Load in all of the components
local filetree = {}
recursiveEnumerate("code/src/components", filetree)
global_components = {}
for i, v in pairs(filetree) do
	local file = love.filesystem.newFile( v )
	require(v:match("(.+)%..+"))
	local name = v:match("^.+/(.+)%..+$")
	global_components[name] = Component.load({name})
end


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
GameObjectiveSystem = require("code/src/systems/controllers/GameObjectiveSystem")

--Events
require("code/src/events/KeyPressed")
require("code/src/events/MousePressed")
require("code/src/events/DamageOccured")
require("code/src/events/GridSelected")
require("code/src/events/FireEvent")
require("code/src/events/ThrusterEvent")
require("code/src/events/GridMenu")

local MainState = {}

function MainState:init()
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

	local whitelist ={"factory", "armor", "thruster", "weapon", "transfer", "technology"}
	global_component_name_list = {}
	for i,v in pairs(datasets) do
		if helper_functions.has_value(whitelist, v.category) then 
			if v.buildable ~= false then
				global_component_name_list[#global_component_name_list + 1] = i
			end
		end
	end

  game_font = love.graphics.newFont("assets/fonts/GiantRobotArmy-Medium.ttf")
  
	ship_data = helper_functions.openjson("code/src/configs/ships.json") 

  tileset_small = helper_functions.createTileset("assets/images/galactiforge_tilesets/factory_devices.png", 32, 32)
  
	--Global variables
	global_show_resource_count = false
	lock_view_to_ship = true
	global_component_index = 1
	global_component_directions = {0,90,180,270}
	global_component_direction_index = 1
	global_ai_active = true
	global_build_mode = false

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
	eventmanager:addListener("GridMenu", GridProcessorSystem(), GridProcessorSystem().fireEvent)

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
	engine:addSystem(GameObjectiveSystem())

	new_game.create_ships() 
	objective_generator.onslaught()

	global_player_ship = engine:getEntitiesWithComponent("PlayerController")

	--Background
	backgroundImage = love.graphics.newImage("assets/images/space_breaker_asset/Background/stars_texture.png")
	backgroundQuad = love.graphics.newQuad(0, 0, backgroundImage:getWidth()*2,backgroundImage:getHeight()*2,backgroundImage:getWidth(),backgroundImage:getWidth())
	backgroundImage:setWrap("repeat", "repeat")

	width = love.graphics.getWidth()
	height = love.graphics.getHeight()

		bg_img = love.graphics.newImage("assets/images/nebula-fractal-space-png-0.png")
		bg_img2 = love.graphics.newImage("assets/images/space-transparent-png-4.png")
		bg_img3 = love.graphics.newImage("assets/images/nebula-transparent-4.png")
		bg_img_stars = {img = love.graphics.newImage("assets/images/star_PNG76860.png"), positions = {}}
		local x_pos = bg_img_stars.img:getWidth()
		
		for i=1,15,1 do
			x_pos = (x_pos + bg_img_stars.img:getWidth())*love.math.randomNormal()
			local y_pos = bg_img_stars.img:getHeight()
			for b=1,15,1 do
				y_pos = (y_pos + bg_img_stars.img:getHeight())*love.math.randomNormal()
				local pos = {x = x_pos, y = y_pos, r=love.math.randomNormal()}
				table.insert(bg_img_stars.positions, pos)
			end
		end

end

function MainState:update(dt)
  world:update(dt)
  world:setCallbacks(beginContact, endContact, preSolve, postSolve)
  engine:update(dt)
end

function MainState:draw()
  
	playerShipLoc = playerShip:get("PositionPhysics")
	playerShipSprite = playerShip:get("Sprite")

	love.graphics.scale(global_zoom_level)

	

	if global_build_mode then

		local x_loc = playerShipLoc.body:getX()*-1+width*(1/global_zoom_level)/2
		local y_loc = playerShipLoc.body:getY()*-1+height*(1/global_zoom_level)/2
		
		love.graphics.translate(width*(1/global_zoom_level)/2, height*(1/global_zoom_level)/2)
		love.graphics.rotate(playerShipLoc.body:getAngle()*-1)
		love.graphics.translate(-width*(1/global_zoom_level)/2, -height*(1/global_zoom_level)/2)
		love.graphics.translate(x_loc,y_loc)

	elseif lock_view_to_ship then


		local x_loc = playerShipLoc.body:getX() * -1 + width * (1 / global_zoom_level) / 2
		local y_loc = playerShipLoc.body:getY() * -1 + height * (1 / global_zoom_level) / 2

		local x_loc_bg = playerShipLoc.body:getX() + width * (1 / global_zoom_level) / 2
		local y_loc_bg = playerShipLoc.body:getY() + height * (1 / global_zoom_level) / 2


		love.graphics.translate(x_loc, y_loc)

	end

	love.graphics.setColor( 1, 1, 1, 0.5 )
	love.graphics.draw(bg_img, 0, 0, 0, 10, 10)
	love.graphics.draw(bg_img2, -8000, -4000, 0, 6, 6)
	love.graphics.draw(bg_img3, -3000, -10000, 0, 8, 8)
	
	for i,v in pairs(bg_img_stars.positions) do
		love.graphics.draw(bg_img_stars.img,v.x, v.y, v.r, 0.8, 0.8)
	end
	love.graphics.setColor( 0.8, 0.8, 0.8, 1 )

	engine:draw()
end

function MainState:keypressed(key, isrepeat)
  if key == "escape" then Gamestate.switch(MenuState) end
	eventmanager:fireEvent(KeyPressed(key, isrepeat))
end

function MainState:mousepressed(x, y, button)
	eventmanager:fireEvent(MousePressed(x, y, button))
end

function MainState:wheelmoved( dx, dy )
	if global_zoom_level >= 0.1 then
		global_zoom_level = global_zoom_level + dy * 0.1
	else 
		global_zoom_level = 0.1
	end
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

return MainState