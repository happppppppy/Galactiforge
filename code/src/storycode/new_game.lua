local newgame = {}

function newgame.create_ships() 
	playerShip = Entity()
	playerShip:initialize(nil, "PlayerShip")
	playerShip:add(global_components.GridMaster(ship_data["intruder"].starter_grid, ship_data["intruder"], 0.5, 32, 32, 1, 1, true))
	playerShip:add(global_components.PositionPhysics(world,500,600,math.rad(180),"dynamic"))
	playerShip:add(global_components.PlayerController())
	playerShip:add(global_components.Faction("Terran"))
	engine:addEntity(playerShip)
end

return newgame