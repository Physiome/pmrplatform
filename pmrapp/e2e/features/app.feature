@main
Feature: Check that the app can be opened

    Background:
    Scenario: Should see the base application working
        When I open the app
        Then I see the following navigational links
            | Home      |
            | Workspace |
            | Exposure  |
        And I see the link labeled Home is highlighted
