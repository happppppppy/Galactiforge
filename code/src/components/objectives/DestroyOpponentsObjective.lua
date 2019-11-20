local DestroyOpponentsObjective = Component.create("DestroyOpponentsObjective")

function DestroyOpponentsObjective:initialize(no_of_ships, centre_x, centre_y, radius)
  self.ships = {}
  for i=1,no_of_ships,1 do

    opponentShip = Entity()
    opponentShip:add(global_components.GridMaster(ship_data["micro_bandit"].starter_grid, ship_data["micro_bandit"], 0.5, 32, 32, 2, 2))
    opponentShip:add(global_components.PositionPhysics(world, centre_x+i*50, centre_y, 0, "dynamic"))
    opponentShip:add(global_components.AIController())
    opponentShip:add(global_components.Faction("Splorg"))
    engine:addEntity(opponentShip)
    table.insert(self.ships, opponentShip)
  -- opponentShip = Entity()
  -- opponentShip:add(global_components.GridMaster(ship_data["intruder"].starter_grid, ship_data["intruder"], 0.5, 32, 32, 2, 2))
  -- opponentShip:add(global_components.PositionPhysics(world,  -4200, -3350, 0, "dynamic"))
  -- opponentShip:add(global_components.AIController())
  -- opponentShip:add(global_components.Faction("Splorg"))
  -- engine:addEntity(opponentShip)
  end
	-- opponentShip = Entity()
	-- opponentShip:add(global_components.GridMaster(ship_data["micro_bandit"].starter_grid, ship_data["micro_bandit"], 0.5, 32, 32, 2, 2))
	-- opponentShip:add(global_components.PositionPhysics(world, -4000, -3000, 0, "dynamic"))
	-- opponentShip:add(global_components.AIController())
	-- opponentShip:add(global_components.Faction("Splorg"))
	-- engine:addEntity(opponentShip)

end