
let web_root = "https://ple1n.github.io/strprox/"

async function findIndexHtmlFiles(dir: string): Promise<string[]> {
    const results: string[] = [];

    async function traverseDirectory(currentDir: string) {
        for await (const entry of Deno.readDir(currentDir)) {
            const fullPath = `${currentDir}/${entry.name}`;
            if (entry.isDirectory) {
                await traverseDirectory(fullPath); // Recurse into subdirectory
            } else if (entry.isFile && entry.name === "index.html") {
                results.push(fullPath); // Store the path of index.html
            }
        }
    }

    await traverseDirectory(dir);
    return results;
}

const dirPath = "./docs";

let paths = await findIndexHtmlFiles(dirPath);

paths = paths.map(x => x.replace("./docs/", web_root));

console.log(paths);

const filePath = "README.md";
const fileContent = await Deno.readTextFile(filePath);

let index = "";

for (let p of paths) {
    index += `- ${p}\n`
}

const updatedContent = fileContent.replace(
    /## Benchmark reports[\s\S]*?(?=\n##|$)/,
    `## Benchmark reports\n\n${index}`
);


await Deno.writeTextFile(filePath, updatedContent);