local newgame = {}

function newgame.create_ships() 
	playerShip = Entity()
	playerShip:initialize(nil, "PlayerShip")
	playerShip:add(global_components.GridMaster(ship_data["intruder"].starter_grid, ship_data["intruder"], 0.5, 32, 32, 1, 1, true))
	playerShip:add(global_components.PositionPhysics(world,500,600,math.rad(180),"dynamic"))
	playerShip:add(global_components.PlayerController())
	playerShip:add(global_components.Faction("Terran"))
	engine:addEntity(playerShip)

	local x,y = 0,0
	for i=1,1,1 do

		opponentShip = Entity()
		opponentShip:add(global_components.GridMaster(ship_data["intruder"].starter_grid, ship_data["intruder"], 0.5, 32, 32, 2, 2))
		opponentShip:add(global_components.PositionPhysics(world, 2000 + x, 2000 + y, 0, "dynamic"))
		opponentShip:add(global_components.AIController())
		opponentShip:add(global_components.Faction("Splorg"))
		engine:addEntity(opponentShip)

		x = i * 200
		y = i * 200
	end

	opponentShip = Entity()
	opponentShip:add(global_components.GridMaster(ship_data["micro_bandit"].starter_grid, ship_data["micro_bandit"], 0.5, 32, 32, 2, 2))
	opponentShip:add(global_components.PositionPhysics(world, -4000, -3000, 0, "dynamic"))
	opponentShip:add(global_components.AIController())
	opponentShip:add(global_components.Faction("Splorg"))
	engine:addEntity(opponentShip)

end

function newgame.create_environment() 

end

return newgame