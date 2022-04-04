import os, filecmp, toml, shutil

shared_files = []

result = {"files": {}}

start = input("Enter folder name: ")

def compareFiles(path):
    for root, directory, files in os.walk(start):
        for name in files:
            full_path = os.path.join(root, name).replace("\\", "/")
            
            if full_path == path:
                continue
            
            if filecmp.cmp(path, full_path):
                mod_path = path[len(start) + 1:]
                full_mod_path = full_path[len(start) + 1:]
                if mod_path not in result["files"]:
                    result["files"][mod_path] = []
                result["files"][mod_path].append(full_mod_path)
                shared_files.append(full_path)

def makeFolder(path):
    try:
        os.makedirs(path)
    except:
        pass

for root, directory, files in os.walk(start):
    for name in files:
        full_path = os.path.join(root, name).replace("\\", "/")
        if full_path not in shared_files:
            compareFiles(full_path)

output_folder = start + " [Shared]"
makeFolder(output_folder)

for x in result["files"]:
    game_path = os.path.split(x)
    src_path = os.path.join(start, game_path[0], game_path[1])
    dest_path = os.path.join(output_folder, game_path[0])
    makeFolder(dest_path)
    shutil.copy(src_path, dest_path)
    for y in result["files"][x]:
        game_path = os.path.split(y)
        dest_path = os.path.join(output_folder, game_path[0])
        makeFolder(dest_path)
        shutil.copy(src_path, dest_path)

with open(os.path.join(output_folder, "share.toml"), "w") as f:
    toml.dump(result, f)