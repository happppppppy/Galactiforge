local objective_generator = {}

function objective_generator.onslaught(objective_loc_x, objective_loc_y, objective_radius)
  objective = Entity()
  objective:add(global_components.DestroyOpponentsObjective(10,1000,1000,200))
  engine:addEntity(objective)
end


return objective_generator