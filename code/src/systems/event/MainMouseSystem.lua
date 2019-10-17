local grid_functions = require("code/src/systems/grids/grid_functions")
local matrix = require("code/lib/matrix")

local MainMouseSystem = class("MainMouseSystem", System)

local function getPoint(x_pos, y_pos, x_offset, y_offset, grid_master, physics)
  local offset_dist = math.sqrt((x_pos*grid_master.grid_size.width + x_offset)^2 + (y_pos*grid_master.grid_size.height + y_offset)^2)*grid_master.grid_scale
  local offset_angle = math.atan2((y_pos*grid_master.grid_size.height + y_offset),(x_pos*grid_master.grid_size.width + x_offset))
  
  local t = physics.body:getAngle()
  local x = physics.body:getX()+offset_dist*math.cos(t + offset_angle)
  local y = physics.body:getY()+offset_dist*math.sin(t + offset_angle)

  return x,y
end

local function sub(m1, m2)
  m3 = {}

  m3[1] = m1[1] - m2[1]
  m3[2] = m1[2] - m2[2]

  return m3
end

local function dot(m1, m2)
  m3 = {}
  m3 = m1[1] * m2[1] + m1[2] * m2[2]

  return m3
end

local function PointInTriangle(A,B,C,P)
  -- Compute vectors        
  v0 = sub(C,A)
  v1 = sub(B,A)
  v2 = sub(P,A)

  -- Compute dot products
  dot00 = dot(v0, v0)
  dot01 = dot(v0, v1)
  dot02 = dot(v0, v2)
  dot11 = dot(v1, v1)
  dot12 = dot(v1, v2)

  -- Compute barycentric coordinates
  invDenom = 1 / (dot00 * dot11 - dot01 * dot01)
  u = (dot11 * dot02 - dot01 * dot12) * invDenom
  v = (dot00 * dot12 - dot01 * dot02) * invDenom

  -- Check if point is in triangle
  return (u >= 0) and (v >= 0) and (u + v < 1)
end


function MainMouseSystem:fireEvent(event)
  if event.button == 2 or event.button == 3 then
    local x,y = love.graphics.inverseTransformPoint( event.x, event.y )
    local entities = engine:getEntitiesWithComponent("GridMaster")
    local mouse_in_grid = false

    for i,v in pairs(entities) do
      grid_master = v:get("GridMaster")
      physics = v:get("PositionPhysics")


      for row = 1, #grid_master.grid_items do
        for col = 1, #grid_master.grid_items[row] do
          local x_loc = grid_master.grid_specs.allowed_grid.grid_origin.x - col 
          local y_loc = grid_master.grid_specs.allowed_grid.grid_origin.y - row

          local xtl,ytl = getPoint(x_loc, y_loc, grid_master.grid_size.width/2, grid_master.grid_size.height/2, grid_master, physics)
          local xtr,ytr = getPoint(x_loc, y_loc, grid_master.grid_size.width/2 - grid_master.grid_size.width, grid_master.grid_size.height/2, grid_master, physics)
          local  xbl,ybl = getPoint(x_loc, y_loc, grid_master.grid_size.width/2, grid_master.grid_size.height/2 - grid_master.grid_size.height, grid_master, physics)
          local  xbr,ybr = getPoint(x_loc, y_loc, grid_master.grid_size.width/2 - grid_master.grid_size.width, grid_master.grid_size.height/2 - grid_master.grid_size.height, grid_master, physics)
          
          local A = { xtl, ytl}
          local B = { xtr, ytr}
          local C = { xbl, ybl}
          local D = { xbr,ybr }
          local P = { x, y}

          if PointInTriangle(A,B,C,P) or PointInTriangle(B,C,D,P) then
            mouse_in_grid = true 
            if event.button == 2 then
              eventmanager:fireEvent(GridSelected(v, x_loc, y_loc, true))
            elseif event.button == 3 then
              eventmanager:fireEvent(GridSelected(v, x_loc, y_loc, false))
            end
            break 
          end
        end
        if mouse_in_grid then break end
      end
      if mouse_in_grid then break end
    end
  end
end


return MainMouseSystem