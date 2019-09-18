GridSelected = class("GridSelected")

function GridSelected:initialize(parent, x_loc, y_loc, add_grid)
    self.parent = parent
    self.x_loc = x_loc
    self.y_loc = y_loc
    self.add_grid = add_grid
end