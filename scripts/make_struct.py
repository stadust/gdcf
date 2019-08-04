def req_input(msg, accept=None):
    print(msg)
    print('> ', end = '')

    i = input()

    if accept is None:
        return i

    if i.strip() not in accept:
        print("Invalid input, expected on of '" + "', '".join(accept) + "'")

        return req_input(accept)
    return i


data = req_input("Please provide the raw response data from the servers")

while req_input("Does the response (still) contain multiple parts (y/n)?", ["y", "n"]) == "y":
    seperator = req_input("Please input the part seperator")
    parts = data.split(seperator)

    print("The parts are:")

    for index, part in enumerate(parts):
        print(str(index + 1) + ".:", part)

    while True:
        index = req_input("Please provide the index of the part of interest")

        try:
            data = parts[int(index) - 1]
        except IndexError:
            print("Index out of bounds! Please try again")
        except ValueError:
            print("The input you provided wasn't an integer")
        else:
            break

    print("The part of the response data we're now currently looking at is:")
    print(data)

struct_name = req_input("Please provide the name of the struct we're generating")

seperator = req_input("We're now looking at a single object. Please provide the string seperating its fields")
parts = data.split(seperator)

if req_input("Is the object indexed (y/n)?", ['y', 'n']) == 'y':
    indices = sorted(parts[::2], key = lambda x: int(x))
else:
    indices = range(1, len(parts) + 1)

print("Your Rust struct for this object is:")

print("#[derive(Debug)]")
print("pub struct", struct_name, "{")
for index in indices:
    print("\t// TODO: figure this out")
    print("\t/// ## GD Internals")
    print(f"\t/// This value is provided at index `{index}`")
    print(f"\tpub index_{index}: String,"),
print("}")

print("And your primitive parser! invocation is:")
print("parser! {")
print(f"\t{struct_name} => {{")
for index in indices:
    print(f"\t\tindex_{index}(index = {index}),")
print("\t}")
print("}")