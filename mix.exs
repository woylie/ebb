defmodule Ebb.MixProject do
  use Mix.Project

  def project do
    [
      app: :ebb,
      version: "0.1.0",
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      escript: escript(),
      deps: deps(),
      aliases: aliases(),
      preferred_cli_env: [
        "coveralls.github": :test,
        "coveralls.html": :test,
        "coveralls.json": :test
      ]
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger]
    ]
  end

  def escript do
    [main_module: Ebb.CLI]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:credo, "~> 1.7.0", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.4.0", only: [:dev, :test], runtime: false},
      {:excoveralls, "~> 0.10", only: :test},
      {:jason, "~> 1.4"},
      {:tzdata, "~> 1.1"},
      {:yaml_elixir, "~> 2.9"}
    ]
  end

  defp aliases do
    [
      setup: [
        "deps.get",
        "cmd cp -r ./deps/tzdata/priv/release_ets ./.tzdata"
      ],
      build: ["setup", "escript.build"],
      install: ["build", "escript.install ebb"]
    ]
  end
end
