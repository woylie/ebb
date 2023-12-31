defmodule Ebb.DaysOff do
  @moduledoc """
  Defines functions for calculations regarding the days off.
  """

  alias Ebb.Configuration

  @type days_off_summary :: %{allowed: integer, taken: integer, left: integer}

  @doc """
  Takes a year and the configuration and returns a summary of the allowed, taken
  and left vacation days for that year.

  ## Example

      iex> calculate_vacation_days(2082, %Ebb.Configuration{})
      %{allowed: 50, taken: 42, left: 8}
  """
  @spec calculate_vacation_days(integer, Configuration.t()) ::
          days_off_summary()
  def calculate_vacation_days(year, %Configuration{
        allowed_days_off: %{vacation_days: allowed_vacation_days},
        vacation_days: vacation_days
      }) do
    calculate_taken_and_left_days(year, allowed_vacation_days, vacation_days)
  end

  @doc """
  Takes a year and the configuration and returns a summary of the allowed, taken
  and left sick days for that year.

  ## Example

      iex> calculate_sick_days(2082, %Ebb.Configuration{})
      %{allowed: 30, taken: 8, left: 22}
  """
  @spec calculate_vacation_days(integer, Configuration.t()) ::
          days_off_summary()
  def calculate_sick_days(year, %Configuration{
        allowed_days_off: %{sick_days: allowed_sick_days},
        sick_days: sick_days
      }) do
    calculate_taken_and_left_days(year, allowed_sick_days, sick_days)
  end

  defp calculate_taken_and_left_days(year, allowed_days, dates) do
    taken_days = dates |> Enum.map(&day_factor(&1, year)) |> Enum.sum()
    days_left = allowed_days - taken_days
    %{allowed: allowed_days, taken: taken_days, left: days_left}
  end

  defp day_factor({date, description}, year) do
    if date.year == year do
      if String.ends_with?(description, " (h)"),
        do: 0.5,
        else: 1
    else
      0
    end
  end
end
