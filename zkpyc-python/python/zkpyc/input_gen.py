import inspect
import textwrap
from zkpyc.types import Public, Private, Array, field
from dataclasses import fields
from typing import get_type_hints

def contains_field_recursive(arg_type):
    if arg_type == field:
        return True
    elif getattr(arg_type, "__origin__", None) in {Private, Public}:
        inner_type = arg_type.__args__[0]
        return contains_field_recursive(inner_type)
    elif getattr(arg_type, "__origin__", None) == Array:
        inner_type = arg_type.__args__[0]
        return contains_field_recursive(inner_type)
    else:
        return False

def parse_argument_value(value, arg_type, prefix=''):
    if getattr(arg_type, "__origin__", None) in {Private, Public}:
        inner_type = arg_type.__args__[0]
        return parse_argument_value(value, inner_type, f'{prefix}')
    elif arg_type == int:
        return f'({prefix} #x{value:08x})'
    elif arg_type == field:
        return f'({prefix} #f{value})'
    elif arg_type == bool:
        return f'({prefix} true)' if value else f'({prefix} false)'
    elif getattr(arg_type, "__origin__", None) == Array:
        inner_type = arg_type.__args__[0]
        result = []
        if getattr(inner_type, "__origin__", None) == Array:  # Check if it's a nested array
            for i, inner_value in enumerate(value):
                inner_prefix = f'{prefix}.{i}'
                inner_values = parse_argument_value(inner_value, inner_type, inner_prefix)
                result.append(f'{inner_values}')
        else:
            for i, inner_value in enumerate(value):
                inner_prefix = f'{prefix}.{i}'
                inner_values = parse_argument_value(inner_value, inner_type, inner_prefix)
                result.append(f'{inner_values}')

        return '\n'.join(result)
    else:
        return str(value)

def prepare_prover_inputs(argument_names, argument_types, modulus, field_tmp, *args, **kwargs):
    global field
    field = field_tmp

    # Get argument values
    argument_values = args + tuple(kwargs.values())

    # Create a dictionary with argument names, values, and type annotations
    arguments_info = {
        name: (value, argument_types.get(name, None))
        for name, value in zip(argument_names, argument_values)
    }

    # Generate Lisp code
    lisp_code = "(let (\n"
    for name, (value, arg_type) in arguments_info.items():
        parsed_value = parse_argument_value(value, arg_type, name)
        lisp_code += textwrap.indent(parsed_value, "    ") + "\n"
    lisp_code += ")\n    false\n)"

     # Wrap the Lisp code inside (set_default_modulus ...) if any argument type is field
    contains_field = any(contains_field_recursive(arg_type) for arg_type in argument_types.values())
    if contains_field:
        lisp_code = textwrap.indent(lisp_code, "    ") + "\n"
        lisp_code = f"(set_default_modulus {modulus}\n{lisp_code})"

    return lisp_code

def prepare_verifier_inputs(argument_names, argument_types, return_type, modulus, return_value, field_tmp, *args, **kwargs):
    global field
    field = field_tmp

    # Get argument values
    argument_values = args + tuple(kwargs.values())

    # Create a dictionary with argument names, values, and type annotations
    arguments_info = {
        name: (value, argument_types.get(name, None))
        for name, value in zip(argument_names, argument_values)
    }

    # Create a dictionary with return value and type annotation
    return_info = {
        'return': (return_value, return_type)
    }
    
    # Generate Lisp code
    lisp_code = "(let (\n"
    # First parse all public values
    for name, (value, arg_type) in arguments_info.items():
        if getattr(arg_type, "__origin__", None) == Private:
            continue  # Skip the iteration if arg_type is Private
    
        parsed_value = parse_argument_value(value, arg_type, name)
        lisp_code += textwrap.indent(parsed_value, "    ") + "\n"
    # Then parse return value and type
    return_value, return_type = return_info.get('return', (None, None))
    parsed_return = parse_argument_value(return_value, return_type, 'return')
    lisp_code += textwrap.indent(parsed_return, "    ") + "\n"
    # Finally close the lisp
    lisp_code += ")\n    false\n)"
    
    # Wrap the Lisp code inside (set_default_modulus ...) if any argument type is field
    contains_field = any(contains_field_recursive(arg_type) for arg_type in argument_types.values())
    if contains_field:
        lisp_code = textwrap.indent(lisp_code, "    ") + "\n"
        lisp_code = f"(set_default_modulus {modulus}\n{lisp_code})"

    return lisp_code

def get_variable_name(obj, global_vars, local_vars):
    if global_vars is None:
        global_vars = globals()
    if local_vars is None:
        local_vars = locals()
    for name, value in global_vars.items():
        if value is obj:
            return name
    for name, value in local_vars.items():
        if value is obj:
            return name
    return None

def convert_literals(obj, class_type, field_tmp):
    if isinstance(obj, (int, bool)):
        return str(obj)
    elif isinstance(obj, list):
        return '[' + ', '.join(map(lambda x: convert_literals(x, class_type, field_tmp), obj)) + ']'
    elif isinstance(obj, field_tmp):
        return f'field({obj})'
    elif isinstance(obj, class_type):
        fields_str = ', '.join(f'{key}={convert_literals(value, class_type, field_tmp)}' for key, value in obj.__dict__.items())
        return f'{class_type.__name__}({fields_str})'
    else:
        return repr(obj)

def represent_type_annotation(class_type, field_tmp):
    if class_type.__name__ == 'Array':
        inner_type = class_type.__args__[0]
        inner_type_len = class_type.__args__[1]
        inner_str = represent_type_annotation(inner_type, field_tmp)
        return f'Array[{inner_str}, {inner_type_len}]'
    elif class_type == int:
        return 'int'
    elif class_type == bool:
        return 'bool'
    elif class_type == field_tmp:
        return 'field'
    else:
        return f'{class_type.__name__}'

def infer_type_annotation(obj, name, class_type, field_tmp):
    if isinstance(obj, list):
        if obj:
            inner_type = obj[0]
            inner_str = infer_type_annotation(inner_type, name, type(inner_type), field_tmp)
            return f'Array[{inner_str}, {len(obj)}]'
    elif isinstance(obj, int) and not isinstance(obj, bool):
        return 'int'
    elif isinstance(obj, bool):
        return 'bool'
    elif isinstance(obj, field_tmp):
        return 'field'
    elif isinstance(obj, class_type):
        return f'{class_type.__name__}'
    else:
        return 'None'

def str_repr(obj, name, field_tmp):
    type_annotation = infer_type_annotation(obj, name, type(obj), field_tmp)
    literals_str = convert_literals(obj, type(obj), field_tmp)
    output = f'{name}: {type_annotation} = {literals_str}'

    return output

def get_class_source_code(cls, field_tmp):
    class_name = cls.__name__
    field_types = get_type_hints(cls)
    fields_info = [(field.name, represent_type_annotation(field_types[field.name], field_tmp)) for field in fields(cls)]

    class_definition = f"@dataclass\nclass {class_name}:\n"
    for field_name, field_type in fields_info:
        class_definition += f"    {field_name}: {field_type}\n"

    return class_definition

# def params_to_string(params, field_tmp):
#     return convert_literals(params, type(params), field_tmp)

def represent_object(obj, alias=None, field_tmp=None, current_module="__main__", global_vars=None, local_vars=None, is_entry_fct=False):
    module = inspect.getmodule(obj)
    if module is None:
        module_name = None
    else:
        module_name = module.__name__

    # check if object is imported module
    if hasattr(obj, '__class__') and obj.__class__.__name__ == 'module' and module_name is not None:
        # If the object is a module, just return the import statement for the module
        return f"import {module_name}"
    # check if object is locally implemented function or entry function
    elif inspect.isfunction(obj) and (module_name == current_module or is_entry_fct):
        # If the object is a function, return its implementation
        source_lines, _ = inspect.getsourcelines(obj)
        function_impl = ''.join(source_lines)
        return '\n' + function_impl
    # check if object is a list, int, bool or field
    elif isinstance(obj, list) or isinstance(obj, int) or isinstance(obj, bool) or type(obj) is field_tmp:
        # If the object is a list, return its string representation
        obj_variable_name = get_variable_name(obj, global_vars, local_vars)
        obj_repr = str_repr(obj, obj_variable_name, field_tmp)
        return obj_repr
    # check if object is locally defined class
    elif inspect.isclass(obj.__class__) and obj.__class__.__name__ != 'function' and module_name == current_module:
        class_impl = get_class_source_code(obj, field_tmp)
        return '\n' + class_impl
    # check if object is imported class instance
    elif inspect.isclass(obj.__class__) and obj.__class__.__name__ != 'function' and module_name is not None:
        class_variable_name = get_variable_name(obj, global_vars, local_vars)
        class_name = obj.__class__.__name__
        obj_repr = str_repr(obj, class_variable_name, field_tmp)
        # If the object is a class, return an import statement for the class
        if alias is None or class_variable_name == alias:
            class_import_statement = f"from {module_name} import {class_name}\n\n{obj_repr}"
        else:
            class_import_statement = f"from {module_name} import {class_name} as {alias}\n\n{obj_repr}"
        return class_import_statement
    if module_name is not None:
        obj_name = obj.__name__ if hasattr(obj, '__name__') else str(obj)
        alias = get_variable_name(obj, global_vars, local_vars)
        if obj_name == alias:
            # If obj_name is the same as alias, use a simple import statement
            import_statement = f"from {module_name} import {obj_name}"
        else:
            # If alias is provided and different from obj_name, use "as alias" in the import statement
            import_statement = f"from {module_name} import {obj_name} as {alias}"
        return import_statement

def process_includes(obj_list, field_tmp, current_module, global_vars=None, local_vars=None):
    result = []
    
    for item in obj_list:
        if isinstance(item, tuple):
            # If item is a pair, pass it as (obj, alias) to parse_include
            obj, alias = item
            result.append(represent_object(obj, alias, field_tmp, current_module, global_vars, local_vars))
        else:
            # If item is not a pair, pass it as obj to parse_include
            result.append(represent_object(item, None, field_tmp, current_module, global_vars, local_vars))
    
    # Concatenate the results with a newline in between
    return '\n'.join(result)
