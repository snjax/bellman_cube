# Bellman examples 

Here is bellman pedersen hash example, written with ethereum bellman fork, designed by [Matter Labs](https://github.com/matter-labs/).

 
 `main.rs` contains a circuit for the statement "I know `x` such that `pedersen(x) == y`"
 
### Constructing a circuit  

To construct a circuit, first flatten your program into its constituent steps. 

Allocate the variables, then enforce the constraints. 

Enforcing the constraint takes the form of `A * B = C`. (is a linear combination, vectors of all your variables)

The `lc` in the `cs.enforce` function stands for "linear combination", and is an inner product of all the variables with some vector of coefficients.

### Generating Parameters 

These examples use the function `generate_random_parameters` to generate a random set of parameters for testing. For real use cases, these parameters would have to be generated securely, through a multi-party computation. 

### Creating a proof

To create a proof, instantiate a version of the struct that is passed into the circuit, with the inputs to the circuit. 

In these examples, the function `create_random_proof` is used to create a random groth16 proof. 

### Verifying a proof

To verify a proof, prepare the verifying key by passing in `params.vk` to `prepare_verifying_key`. This gives you the prepared viewing key, `pvk`.

The function `verify_proof` takes the prepared viewing key `pvk`, the `proof`, and the output as an array.

## Running 

`cargo build`

`cargo run` runs the example proof in the main file.
