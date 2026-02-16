use openai_agents::Agent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct Recipe {
    name: String,
    ingredients: Vec<String>,
    instructions: Vec<String>,
    prep_time_minutes: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let chef = Agent::builder("Chef")
        .instructions("You are a world-class chef. Provide a recipe for the requested dish.")
        .model("gpt-4o-mini")
        .output_type::<Recipe>()
        .build();

    println!("Requesting a recipe for 'Chocolate Cake'...");

    let result =
        openai_agents::Runner::run(&chef, "Tell me how to make a simple chocolate cake.").await?;

    println!("\n--- Final Text Output ---");
    println!("{}", result.final_output());

    println!("\n--- Parsed Structured Output ---");
    match result.final_output_as::<Recipe>() {
        Ok(recipe) => {
            println!("Recipe Name: {}", recipe.name);
            println!("Prep Time: {} mins", recipe.prep_time_minutes);
            println!("Ingredients: {}", recipe.ingredients.join(", "));
            println!("Instructions:");
            for (i, step) in recipe.instructions.iter().enumerate() {
                println!("{}. {}", i + 1, step);
            }
        }
        Err(e) => println!("Failed to parse recipe: {}", e),
    }

    Ok(())
}
