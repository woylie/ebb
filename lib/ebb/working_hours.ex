defmodule Ebb.WorkingHours do
  @moduledoc """
  Defines functions for calculating working hours.
  """

  alias Ebb.Configuration

  @seconds_per_hour 3600

  @doc """
  Takes an end date (usually today) and returns the number of expected work
  seconds since the start date from the configuration.

  Considers working days, vacation days, sick days and holidays in the
  calculation.
  """
  @spec calculate_expected_work_seconds(Date.t(), Configuration.t()) :: float
  def calculate_expected_work_seconds(
        end_date,
        %Configuration{
          start_date: start_date,
          time_adjustment_in_seconds: time_adjustment_in_seconds,
          working_days: working_days
        } = config
      ) do
    {full_weeks, remaining_days} =
      calculate_weeks_and_days(end_date, start_date)

    full_week_hours = calculate_full_week_hours(full_weeks, working_days)

    remaining_days_hours =
      calculate_remaining_days_hours(remaining_days, end_date, working_days)

    days_off = calculate_hours_off(end_date, config)

    time_adjustment_in_hours = time_adjustment_in_seconds / @seconds_per_hour

    (full_week_hours + remaining_days_hours - time_adjustment_in_hours -
       days_off) *
      @seconds_per_hour
  end

  defp calculate_weeks_and_days(end_date, start_date) do
    days_diff = Date.diff(end_date, start_date) + 1
    full_weeks = div(days_diff, 7)
    remaining_days = rem(days_diff, 7)
    {full_weeks, remaining_days}
  end

  defp calculate_full_week_hours(number_of_weeks, working_days) do
    working_hours_per_week = working_days |> Map.values() |> Enum.sum()
    number_of_weeks * working_hours_per_week
  end

  defp calculate_remaining_days_hours(0, _, _), do: 0

  defp calculate_remaining_days_hours(remaining_days, today, working_days) do
    start_date = Date.add(today, -remaining_days + 1)
    range = Date.range(start_date, today)

    Enum.reduce(range, 0, fn date, hours ->
      hours + get_hours_for_day(date, working_days)
    end)
  end

  defp calculate_hours_off(
         end_date,
         %Configuration{start_date: start_date, working_days: working_days} =
           config
       ) do
    config
    |> get_days_off()
    |> filter_dates_in_range(start_date, end_date)
    |> Enum.map(&get_hours_for_day(&1, working_days))
    |> Enum.sum()
  end

  defp get_hours_for_day(date, working_days) do
    {date, half_or_full} =
      case date do
        {_, _} = v -> v
        d -> {d, :full}
      end

    hours = Map.fetch!(working_days, Date.day_of_week(date))
    if half_or_full == :half, do: 0.5 * hours, else: hours
  end

  defp get_days_off(%Configuration{
         holidays: holidays,
         sick_days: sick_days,
         vacation_days: vacation_days
       }) do
    holidays
    |> Map.merge(sick_days)
    |> Map.merge(vacation_days)
    |> Enum.map(fn
      {date, description} ->
        if String.ends_with?(description, " (h)"),
          do: {date, :half},
          else: {date, :full}
    end)
  end

  defp filter_dates_in_range(dates, start_date, end_date) do
    Enum.reject(dates, fn {date, _} ->
      Date.compare(date, start_date) == :lt or
        Date.compare(date, end_date) == :gt
    end)
  end
end
