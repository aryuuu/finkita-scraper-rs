use chrono::prelude::*;
// use reqwest;
use dotenv::dotenv;
use std::{thread, time};
use thirtyfour::prelude::*;
// use tokio;
mod config;
mod lib;
mod repo;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    dotenv().ok();
    let bni_config = config::BNIConfig::init();
    let mut capabilities = DesiredCapabilities::chrome();
    capabilities.set_headless()?;
    capabilities.set_ignore_certificate_errors()?;
    // capabilities.set_native_events()?;
    // let driver = WebDriver::new("http://localhost:4444/wd/hub", &capabilities).await?;
    let driver = WebDriver::new("http://localhost:4444", &capabilities).await?;
    println!("driver initialized");
    // let driver = WebDriver::new("http://localhost:4445", &capabilities).await?;

    // Navigate to url
    driver.get(&bni_config.ibank_url).await?;
    println!("home page loaded");

    login(&driver, bni_config).await?;
    open_accounts_page(&driver).await?;
    open_mutations_page(&driver).await?;
    open_opr_page(&driver).await?;
    // open_full_statement_page(&driver).await?;
    open_current_month_statement_page(&driver).await?;

    let mut mutations = generate_empty_mutation_vec(&driver).await?;
    let mut offset = 0;

    'extracting_statement: loop {
        extract_mutation_date(&driver, &mut mutations, &offset).await?;
        extract_mutation_description(&driver, &mut mutations, &offset).await?;
        extract_mutation_type(&driver, &mut mutations, &offset).await?;
        extract_mutation_amount(&driver, &mut mutations, &offset).await?;
        extract_balance(&driver, &mut mutations, &offset).await?;

        println!("mutations: {:#?}", mutations);

        let next_page_button_xpath = "//a[@id='NextData']";
        let next_page_button = driver.find_element(By::XPath(next_page_button_xpath)).await;

        match next_page_button {
            Ok(next_page_button) => {
                println!("Found the next button");
                println!("Navigating to the next page..");
                next_page_button.click().await?;
            }
            Err(_error) => {
                println!("Next page button not found");
                println!("Breaking loop...");
                break 'extracting_statement;
            }
        }

        thread::sleep(time::Duration::from_millis(500));
        offset += 10;
    }

    // check at the loaded page for a moment
    thread::sleep(time::Duration::from_secs(3));
    logout(&driver).await?;

    // bulk_write_mutation(&mutations).await?;

    driver.quit().await?;

    Ok(())
}

async fn login(driver: &WebDriver, bni_config: config::BNIConfig) -> WebDriverResult<()> {
    println!("login");
    let user_id_input_id = "CorpId";
    let password_input_id = "PassWord";
    let login_button_xpath = "//input[@type='submit']";

    let user_id_text_input = driver.find_element(By::Id(user_id_input_id)).await?;
    let password_text_input = driver.find_element(By::Id(password_input_id)).await?;
    let login_button = driver.find_element(By::XPath(login_button_xpath)).await?;

    user_id_text_input.send_keys(bni_config.username).await?;
    password_text_input.send_keys(bni_config.password).await?;
    login_button.click().await?;

    // wait for login process to finish
    thread::sleep(time::Duration::from_secs(1));

    Ok(())
}

async fn open_accounts_page(driver: &WebDriver) -> WebDriverResult<()> {
    let cls_link_content_xpath = "//div[@class='clsLinkContent']";
    let first_cls_link_content = driver
        .find_element(By::XPath(cls_link_content_xpath))
        .await?;

    // get into ACCOUNTS page
    first_cls_link_content.click().await?;
    // wait for ACCOUNTS page
    thread::sleep(time::Duration::from_millis(500));
    Ok(())
}

async fn open_mutations_page(driver: &WebDriver) -> WebDriverResult<()> {
    let account_menu_list_xpath = "//a[@id='AccountMenuList']";
    let account_menu_list = driver
        .find_elements(By::XPath(account_menu_list_xpath))
        .await?;
    // go into mutation page
    let mutation_nav_button = &account_menu_list[2];
    mutation_nav_button.click().await?;
    // wait for mutation page
    thread::sleep(time::Duration::from_millis(500));
    Ok(())
}

async fn open_opr_page(driver: &WebDriver) -> WebDriverResult<()> {
    // dropdown value: OPR
    let main_account_type_dropdown_id = "MAIN_ACCOUNT_TYPE";
    let main_account_type_dropdown = driver
        .find_element(By::Id(main_account_type_dropdown_id))
        .await?;

    main_account_type_dropdown.click().await?;

    let opr_option_xpath = "//option[@value='OPR']";
    let opr_option = driver.find_element(By::XPath(opr_option_xpath)).await?;
    let account_id_select_button_id = "AccountIDSelectRq";
    let account_id_select_button = driver
        .find_element(By::Id(account_id_select_button_id))
        .await?;

    opr_option.click().await?;
    account_id_select_button.click().await?;
    thread::sleep(time::Duration::from_millis(500));
    Ok(())
}

// async fn open_full_statement_page(driver: &WebDriver) -> WebDriverResult<()> {
//     let txn_period_dropdown_id = "TxnPeriod";
//     let txn_period_dropdown = driver.find_element(By::Id(txn_period_dropdown_id)).await?;
//     txn_period_dropdown.click().await?;

//     let last_month_option_xpath = "//option[@value='LastMonth']";
//     let last_month_option = driver
//         .find_element(By::XPath(last_month_option_xpath))
//         .await?;
//     last_month_option.click().await?;

//     let next_button_id = "FullStmtInqRq";
//     let next_button = driver.find_element(By::Id(next_button_id)).await?;
//     next_button.click().await?;
//     Ok(())
// }

async fn open_current_month_statement_page(driver: &WebDriver) -> WebDriverResult<()> {
    let txn_date_radio_button_xpath = "//input[@id='Search_Option_6']";
    let txn_date_radio_button = driver
        .find_element(By::XPath(txn_date_radio_button_xpath))
        .await?;
    txn_date_radio_button.click().await?;

    let start_date_input_text_xpath = "//input[@id='txnSrcFromDate']";
    let end_date_input_text_xpath = "//input[@id='txnSrcToDate']";
    let start_date_input_text = driver
        .find_element(By::XPath(start_date_input_text_xpath))
        .await?;
    let end_date_input_text = driver
        .find_element(By::XPath(end_date_input_text_xpath))
        .await?;
    let now = Utc::now();
    start_date_input_text.clear().await?;
    start_date_input_text
        .send_keys(get_current_month_start_date(&now))
        .await?;

    end_date_input_text.clear().await?;
    end_date_input_text
        .send_keys(get_current_date(&now))
        .await?;

    let next_button_id = "FullStmtInqRq";
    let next_button = driver.find_element(By::Id(next_button_id)).await?;
    next_button.click().await?;

    Ok(())
}

async fn generate_empty_mutation_vec(driver: &WebDriver) -> WebDriverResult<Vec<lib::Mutation>> {
    let num_of_statements: usize;
    let result_display_string_tag_id = "ResultsDisplayStringTag";
    let result_display_string_tag = driver
        .find_element(By::Id(result_display_string_tag_id))
        .await;
    match result_display_string_tag {
        Ok(result_display_string_tag) => {
            let result_display_string = result_display_string_tag.text().await?;
            let num_of_statements_string = result_display_string
                .split_whitespace()
                .next_back()
                .unwrap();
            num_of_statements = num_of_statements_string.parse::<usize>().unwrap();
        }
        Err(_error) => {
            let mutation_descriptions_xpath = "//td[@id='s2']/span[@id='H']";
            let mutation_descriptions = driver
                .find_elements(By::XPath(mutation_descriptions_xpath))
                .await?;
            num_of_statements = mutation_descriptions.len();
        }
    }

    let result: Vec<lib::Mutation> = (0..num_of_statements)
        .map(|_| lib::Mutation::new())
        .collect();

    Ok(result)
}

async fn extract_mutation_date(
    driver: &WebDriver,
    mutations: &mut Vec<lib::Mutation>,
    offset: &usize,
) -> WebDriverResult<()> {
    let mutation_dates_xpath = "//td[@id='s1']/span[@id='H']";
    let mutation_dates = driver
        .find_elements(By::XPath(mutation_dates_xpath))
        .await?;
    let mut idx = 0;

    for mutation_date in mutation_dates {
        let date = mutation_date.text().await?;
        if !date.is_empty() {
            let date_text = mutation_date.text().await?;
            let naive_date = NaiveDate::parse_from_str(&date_text, "%d-%b-%Y").unwrap();
            let naive_time = NaiveTime::from_hms(0, 0, 0);
            let naive_date_time = NaiveDateTime::new(naive_date, naive_time);
            let tz_offset = FixedOffset::east(7 * 3600);

            let dt_with_tz = tz_offset.from_local_datetime(&naive_date_time).unwrap();
            let dt_with_tz_utc = Utc.from_utc_datetime(&dt_with_tz.naive_utc());

            mutations.get_mut(idx + offset).unwrap().date = dt_with_tz_utc;
            idx += 1;
        }
    }

    Ok(())
}

async fn extract_mutation_description(
    driver: &WebDriver,
    mutations: &mut Vec<lib::Mutation>,
    offset: &usize,
) -> WebDriverResult<()> {
    let mutation_descriptions_xpath = "//td[@id='s2']/span[@id='H']";
    let mutation_descriptions = driver
        .find_elements(By::XPath(mutation_descriptions_xpath))
        .await?;

    for (idx, desc) in mutation_descriptions.iter().enumerate() {
        let desc_text = desc.text().await?;
        mutations.get_mut(idx + offset).unwrap().description = desc_text;
    }

    Ok(())
}

async fn extract_mutation_type(
    driver: &WebDriver,
    mutations: &mut Vec<lib::Mutation>,
    offset: &usize,
) -> WebDriverResult<()> {
    // special case for type, since account number share the same xpath with it
    let mutation_types_xpath = "(//td[@id='s3']/span[@id='H'])[position()>1]";
    let mutation_types = driver
        .find_elements(By::XPath(mutation_types_xpath))
        .await?;

    for (idx, mutation_type) in mutation_types.iter().enumerate() {
        let type_text = mutation_type.text().await?;
        mutations.get_mut(idx + offset).unwrap().m_type = type_text;
    }

    Ok(())
}

async fn extract_mutation_amount(
    driver: &WebDriver,
    mutations: &mut Vec<lib::Mutation>,
    offset: &usize,
) -> WebDriverResult<()> {
    let mutation_amounts_xpath = "//td[@id='s4']/span[@id='H']";
    let mutation_amounts = driver
        .find_elements(By::XPath(mutation_amounts_xpath))
        .await?;

    for (idx, mutation_amount) in mutation_amounts.iter().enumerate() {
        let amount_text = mutation_amount.text().await?;
        let amount = parse_idr_to_i32(&amount_text);
        mutations.get_mut(idx + offset).unwrap().amount = amount;
    }

    Ok(())
}

async fn extract_balance(
    driver: &WebDriver,
    mutations: &mut Vec<lib::Mutation>,
    offset: &usize,
) -> WebDriverResult<()> {
    let balances_xpath = "//td[@id='s5']/span[@id='H']";
    let balances = driver.find_elements(By::XPath(balances_xpath)).await?;

    for (idx, balance) in balances.iter().enumerate() {
        let balance_text = balance.text().await?;
        let balance = parse_idr_to_i32(&balance_text);
        mutations.get_mut(idx + offset).unwrap().balance = balance;
    }

    Ok(())
}

async fn logout(driver: &WebDriver) -> WebDriverResult<()> {
    let logout_button_id = "LogOut";
    let logout_button = driver.find_element(By::Id(logout_button_id)).await?;

    logout_button.click().await?;

    // wait for confirmation page
    thread::sleep(time::Duration::from_millis(500));

    let logout_confirmation_id = "__LOGOUT__";
    let logout_confirmation_button = driver.find_element(By::Id(logout_confirmation_id)).await?;

    logout_confirmation_button.click().await?;

    Ok(())
}

async fn bulk_write_mutation(mutations: &Vec<lib::Mutation>) -> WebDriverResult<()> {
    let finkita_service_url = "http://localhost:7878";
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/mutations", finkita_service_url))
        .json(mutations)
        .send()
        .await?;

    println!("{:?}", res.status());

    Ok(())
}

fn get_current_month_start_date(now: &DateTime<Utc>) -> String {
    now.format("01-%b-%Y").to_string()
}

fn get_current_date(now: &DateTime<Utc>) -> String {
    now.format("%d-%b-%Y").to_string()
}

fn parse_idr_to_i32(idr: &str) -> i32 {
    let idr = &idr[4..idr.len() - 3];
    let idr = idr.replace(".", "");
    let amount = idr.parse::<i32>().unwrap();
    amount
}
