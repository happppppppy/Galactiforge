local helper_functions = { _version = "0.0.1" }

local json = require "code/lib/json"

function helper_functions.openjson(path)      
	-- will hold contents of file
	local contents
	-- io.open opens a file at path. returns nil if no file found
	local file = io.open( path, "r" )
	if file then
	   -- read all contents of file into a string
	   contents = file:read( "*a" )
	   io.close( file )     -- close the file after using it
	else
			print( "** Error: cannot open file" )
	end    

	local jsonTable=json.decode(contents)	
	return jsonTable
end

function helper_functions.deepcopy(orig)
	local orig_type = type(orig)
	local copy
	if orig_type == 'table' then
			copy = {}
			for orig_key, orig_value in next, orig, nil do
					copy[deepcopy(orig_key)] = deepcopy(orig_value)
			end
			setmetatable(copy, deepcopy(getmetatable(orig)))
	else -- number, string, boolean, etc
			copy = orig
	end
	return copy
end

function helper_functions.createTileset(image_path, tile_width, tile_height)
	tileset = {}
	tileset.image = love.graphics.newImage(image_path)
	tileset.tile_width = tile_width
	tileset.tile_height = tile_height
	tileset.num_tiles_x = tileset.image:getWidth()/tileset.tile_width
	tileset.num_tiles_y =  tileset.image:getHeight()/tileset.tile_height
	tileset.tiles = {}
	local index = tileset.num_tiles_x * tileset.num_tiles_y
	for y = tileset.num_tiles_y, 1, -1 do
		for x = tileset.num_tiles_x, 1, -1 do
			tileset.tiles[index] = love.graphics.newQuad(tileset.tile_width * (x-1), tileset.tile_height * (y-1), tileset.tile_width, tileset.tile_height, tileset.image:getDimensions())
			index = index - 1
		end
	end
	return tileset
end

return helper_functions